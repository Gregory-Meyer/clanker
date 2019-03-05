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
