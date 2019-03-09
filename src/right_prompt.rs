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

extern crate libgit2_sys;

mod color;
mod git;

use git::Repository;

use std::{env, process::Command, thread};

use color::Color;

fn main() {
    print!("{}", make_prompt());
}

fn make_prompt() -> String {
    let args: Vec<_> = env::args().collect();

    let retc = match args.get(1) {
        Some(r) => r.parse().unwrap_or(0),
        None => 0,
    };

    if let Some(head) = repo_head() {
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

macro_rules! try_option {
    ($x:expr) => {
        match $x {
            Some(x) => x,
            None => return None,
        }
    };
}

fn repo_head() -> Option<String> {
    let is_dirty_thread = thread::spawn(repository_is_dirty);

    let repo = try_option!(Repository::open_from_env());
    let identifier = try_option!(identify_head(&repo));

    if is_dirty_thread.join().ok().unwrap_or(false) {
        Some(format!("{} *", identifier))
    } else {
        Some(identifier)
    }
}

fn identify_head(repo: &Repository) -> Option<String> {
    let head = try_option!(repo.head());

    if let Some(name) = head.branch_name() {
        Some(name.to_string_lossy().into_owned())
    } else {
        let head_commit = try_option!(head.peel_to_commit()); // this had better point to a commit...
        let tags = repo
            .tags_pointing_to(&head_commit)
            .unwrap_or_else(|| Vec::new());

        if tags.is_empty() {
            head_commit.as_object().short_id()
        } else {
            let tag_names: Vec<_> = tags.iter().map(|n| n.to_string_lossy()).collect();

            Some(tag_names.join("\\"))
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
