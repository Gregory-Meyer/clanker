extern crate colored;
extern crate dirs;
extern crate nix;
extern crate unicode_segmentation;
extern crate whoami;

use std::{env, iter, path::PathBuf};

use colored::Colorize;
use nix::unistd::{self, Uid};
use unicode_segmentation::UnicodeSegmentation;

fn main() {
    println!("{}", make_prompt());
}

fn make_prompt() -> String {
    let cwd = compress(cwd());

    if unistd::geteuid() == Uid::from_raw(0) {
        format!("{}@{} {}# ", whoami::username(), whoami::hostname(), cwd.red())
    } else {
        format!("{}@{} {}> ", whoami::username(), whoami::hostname(), cwd.green())
    }
}

fn cwd() -> String {
    let mut cwd = match env::current_dir() {
        Ok(p) => p,
        Err(_) => return "?".to_string(),
    };

    if let Some(home) = dirs::home_dir() {
        if home == cwd {
            return "~".to_string();
        }

        if let Ok(home) = cwd.strip_prefix(home) {
            cwd = PathBuf::from("~").join(home);
        }
    }

    format!("{}", cwd.display())
}

fn compress(path: String) -> String {
    let separator = if cfg!(windows) {
        "\\"
    } else {
        "/"
    };

    let components: Vec<&str> = path.split(separator).collect();
    let (last, rest) = components.split_last().unwrap();

    if rest.is_empty() {
        return last.to_string();
    }

    let parts: Vec<&str> = rest.iter().map(|s| {
        let mut graphemes = s.grapheme_indices(true);
        match graphemes.next() {
            Some((_, g)) => if g == "." {
                graphemes.next().map(|(j, h)| &s[..j + h.len()]).unwrap_or(g)
            } else {
                g
            }
            None => "",
        }
    }).chain(iter::once(*last))
        .collect();

    parts.join(separator)
}
