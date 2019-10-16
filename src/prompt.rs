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

mod color;
mod compress;

use color::Color;
use compress::IntoStringLossy;

use std::{env, ffi::CStr, mem::MaybeUninit};

fn main() {
    let cwd = compress::cwd().unwrap_or_else(|_| "?".to_string());

    let (cursor, root_cursor) = cursors();
    let (username, is_root) = username_and_is_root();

    if is_root {
        print!("{}@{} {}{} ", username, hostname(), cwd.red(), root_cursor)
    } else {
        print!("{}@{} {}{} ", username, hostname(), cwd.green(), cursor)
    }
}

fn cursors() -> (String, String) {
    let mut iter = env::args_os();

    iter.next(); // skip first arg, the exec name

    let cursor = iter
        .next()
        .map(|c| c.into_string_lossy())
        .unwrap_or_else(|| ">".to_string());
    let root_cursor = iter
        .next()
        .map(|c| c.into_string_lossy())
        .unwrap_or_else(|| "#".to_string());

    (cursor, root_cursor)
}

fn username_and_is_root() -> (String, bool) {
    let euid = unsafe { libc::geteuid() };
    let passwd = unsafe { libc::getpwuid(euid) };
    assert!(!passwd.is_null());

    let username = unsafe { CStr::from_ptr((*passwd).pw_name) };

    (username.to_string_lossy().into_owned(), euid == 0)
}

fn hostname() -> String {
    // less work than utsname - no dynamic alloc for hostname buffer
    // probably
    let mut utsname = unsafe { MaybeUninit::uninit().assume_init() };
    let ret = unsafe { libc::uname(&mut utsname) };
    assert_eq!(ret, 0);

    let hostname = unsafe { CStr::from_ptr(utsname.nodename.as_ptr()) };

    hostname.to_string_lossy().into_owned()
}
