extern crate colored;
extern crate dirs;
extern crate nix;
extern crate unicode_segmentation;
extern crate whoami;

mod compress;

use colored::Colorize;
use nix::unistd::{self, Uid};

fn main() {
    println!("{}", make_prompt());
}

fn make_prompt() -> String {
    let cwd = compress::compressed_cwd();

    if unistd::geteuid() == Uid::from_raw(0) {
        format!("{}@{} {}# ", whoami::username(), whoami::hostname(), cwd.red())
    } else {
        format!("{}@{} {}> ", whoami::username(), whoami::hostname(), cwd.green())
    }
}
