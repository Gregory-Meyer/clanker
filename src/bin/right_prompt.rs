extern crate colored;
extern crate git2;

use std::env;

use colored::Colorize;
use git2::{Branch, Repository};

fn main() {
    println!("{}", make_prompt());
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

fn repo_head() -> Option<String> {
    let repo = match Repository::open_from_env() {
        Ok(r) => r,
        Err(_) => return None,
    };

    let has_changes = repo.diff_tree_to_workdir_with_index(None, None)
        .map(|d| d.deltas().next().is_some())
        .unwrap_or(false);

    let head = repo.head().unwrap();

    if head.is_branch() {
        let branch_name = Branch::wrap(head)
            .name()
            .unwrap()
            .map(|n| n.to_string())
            .unwrap_or("?".to_string());

        if has_changes {
            Some(format!("{} *", branch_name))
        } else {
            Some(branch_name)
        }
    } else {
        let mut hash = head.peel_to_commit()
            .unwrap()
            .id()
            .to_string();

        hash.truncate(7);

        if has_changes {
            Some(format!("{} *", hash))
        } else {
            Some(hash)
        }
    }
}
