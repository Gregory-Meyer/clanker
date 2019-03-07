// MIT License
//
// Copyright (c) 2019 Gregory Meyer
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

extern crate colored;
extern crate git2;

use std::{env, process::Command, thread};

use colored::Colorize;
use git2::{Branch, Commit, Error, Repository};

fn main() {
    print!("{}", make_prompt());
}

fn make_prompt() -> String {
    let args: Vec<_> = env::args().collect();

    let retc = match args.get(1) {
        Some(r) => r.parse().unwrap_or(0),
        None => 0,
    };

    if let Ok(head) = repo_head() {
        if retc != 0 {
            format!("{} ({})", retc.to_string().red(), head)
        } else {
            format!("({})", head)
        }
    } else {
        if retc != 0 {
            format!("{}", retc.to_string().red())
        } else {
            "".to_string()
        }
    }
}

fn repo_head() -> Result<String, Error> {
    let is_dirty_thread = thread::spawn(repository_is_dirty);
    let repo = Repository::open_from_env()?;
    let identifier = identify_head(&repo)?;

    if is_dirty_thread.join().ok().unwrap_or(false) {
        Ok(format!("{} *", identifier))
    } else {
        Ok(identifier)
    }
}

fn identify_head(repo: &Repository) -> Result<String, Error> {
    let head = repo.head()?;

    if head.is_branch() {
        let branch = Branch::wrap(head);

        Ok(String::from_utf8_lossy(branch.name_bytes()?).into_owned())
    } else {
        let head_commit = head.peel_to_commit()?; // this had better point to a commit...

        let mut refname_buf = "refs/tags/".to_string(); // reuse on each iteration
        let tags = repo
            .tag_names(None)?
            .iter() // this irks me - git2-rs devs have decided only UTF-8 tags are allowed
            .filter_map(|n| n)
            .filter(|n| tag_points_to_commit(&repo, n, &head_commit, &mut refname_buf))
            .fold(String::new(), append_tag_name);

        if tags.is_empty() {
            let mut id = head_commit.id().to_string();
            id.truncate(7);

            Ok(id)
        } else {
            Ok(tags)
        }
    }
}

fn repository_is_dirty() -> bool {
    // this is much faster than checking for the first diff and then aborting
    // difference on my computer was from 3s (hand rolled libgit2, abort after first diff) to
    // 660 ms when using the future-style computation here
    Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .ok()
        .map(|output| {
            if !output.status.success() {
                return false;
            }

            !output.stdout.is_empty()
        })
        .unwrap_or(false)
}

fn tag_points_to_commit(
    repo: &Repository,
    tag_name: &str,
    target: &Commit,
    buf: &mut String,
) -> bool {
    buf.push_str(tag_name);

    let id = repo
        .find_reference(&buf)
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id();

    buf.truncate("refs/tags/".len());

    id == target.id()
}

fn append_tag_name(mut tags: String, tag_name: &str) -> String {
    if !tags.is_empty() {
        // backslash is one of few disallowed characters in git identifiers
        // so we use it to delimit multiple tags
        tags.push('\\');
    }

    tags.push_str(tag_name);

    tags
}
