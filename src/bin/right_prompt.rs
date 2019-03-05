extern crate colored;
extern crate libc;
extern crate libgit2_sys;

mod git;

use git::{DiffOptions, Repository};

use std::env;

use colored::Colorize;
use libc::{c_char, c_int, c_void};
use libgit2_sys::{git_diff, git_diff_delta};

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

fn repo_head() -> Option<String> {
    let mut repo = match Repository::open_from_env() {
        Ok(r) => r,
        Err(_) => return None,
    };

    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return None,
    };

    let head_commit = head.peel_to_commit().expect("couldn't peel HEAD to commit");
    let mut head_tree = head_commit
        .tree()
        .expect("couldn't get tree from HEAD commit");

    let mut has_diff = false;

    let options = DiffOptions::new()
        .include_untracked()
        .skip_binary_check()
        .enable_fast_untracked_dirs()
        .set_notify_cb(notify_cb)
        .set_payload(&mut has_diff as *mut _ as *mut c_void);

    match repo.diff_tree_to_workdir_with_index(Some(&mut head_tree), Some(&options)) {
        _ => (),
    };

    if let Ok(name) = head.branch_name().map(|n| n.to_string_lossy()) {
        if has_diff {
            Some(format!("{} *", name))
        } else {
            Some(name.into_owned())
        }
    } else {
        let mut oid = head_commit.id().to_string();
        oid.truncate(7);

        if has_diff {
            Some(format!("{} *", oid))
        } else {
            Some(oid)
        }
    }
}

extern "C" fn notify_cb(
    _: *const git_diff,
    _: *const git_diff_delta,
    _: *const c_char,
    has_changes: *mut c_void,
) -> c_int {
    unsafe { *(has_changes as *mut bool) = true };

    -1 // stop diff iteration
}
