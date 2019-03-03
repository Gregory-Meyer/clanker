extern crate dirs;
extern crate git2;

use std::{env, path::PathBuf};

use git2::{Branch, Repository};

fn main() {
    println!("{}", make_prompt());
}

fn make_prompt() -> String {
    let cwd = cwd();

    if let Some(branch) = repo_head() {
        format!("{} ({}) ~> ", cwd, branch)
    } else {
        format!("{} ~>", cwd)
    }
}

fn cwd() -> String {
    if cfg!(windows) {
        match env::current_dir() {
            Ok(p) => format!("{}", p.display()),
            Err(_) => "?".to_string(),
        }
    } else {
        let mut cwd = match env::current_dir() {
            Ok(p) => p,
            Err(_) => return "?".to_string(),
        };

        if let Some(p) = dirs::home_dir() {
            if let Ok(p) = cwd.strip_prefix(p) {
                cwd = PathBuf::from("~").join(p.to_path_buf());
            }
        }

        format!("{}", cwd.display())
    }
}

fn repo_head() -> Option<String> {
    let repo = match Repository::open_from_env() {
        Ok(r) => r,
        Err(_) => return None,
    };

    let head = repo.head().unwrap();

    if head.is_branch() {
        let branch_name = Branch::wrap(head)
            .name()
            .unwrap()
            .map(|n| n.to_string())
            .unwrap_or("?".to_string());

        Some(branch_name)
    } else {
        let mut hash = head.peel_to_commit()
            .unwrap()
            .id()
            .to_string();

        hash.truncate(7);

        Some(hash)
    }
}
