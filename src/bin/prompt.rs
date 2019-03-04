extern crate colored;
extern crate dirs;
extern crate nix;
extern crate unicode_segmentation;
extern crate whoami;

mod compress;

use colored::Colorize;
use nix::unistd::{self, Uid};

fn main() {
    let cwd = compress::compressed_cwd();

    if is_current_user_priviliged() {
        print!(
            "{}@{} {}# ",
            whoami::username(),
            whoami::hostname(),
            cwd.red()
        )
    } else {
        print!(
            "{}@{} {}> ",
            whoami::username(),
            whoami::hostname(),
            cwd.green()
        )
    }
}

fn is_current_user_priviliged() -> bool {
    unistd::geteuid() == Uid::from_raw(0)
}
