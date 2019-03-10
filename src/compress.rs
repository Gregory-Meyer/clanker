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

use std::{
    env,
    ffi::{CStr, OsStr, OsString},
    io,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    ptr::NonNull,
};

use libc::{c_char, passwd};
use unicode_segmentation::UnicodeSegmentation;

pub trait IntoStringLossy {
    fn into_string_lossy(self) -> String;
}

impl IntoStringLossy for OsString {
    fn into_string_lossy(self) -> String {
        self.into_string()
            .unwrap_or_else(|s| s.to_string_lossy().into_owned())
    }
}

impl IntoStringLossy for PathBuf {
    fn into_string_lossy(self) -> String {
        self.into_os_string().into_string_lossy()
    }
}

pub fn cwd() -> Result<String, io::Error> {
    current_dir()
        .map(|p| {
            p.into_os_string()
                .into_string()
                .unwrap_or_else(|s| s.to_string_lossy().into_owned())
        })
        .map(compress)
}

fn current_dir() -> Result<PathBuf, io::Error> {
    if let Some(path) = env::var_os("PWD") {
        Ok(PathBuf::from(path))
    } else {
        env::current_dir()
    }
}

fn compress(path: String) -> String {
    let home_compressed = compress_home_prefix(path);
    let (components, last) = match home_compressed.rfind('/') {
        Some(i) => home_compressed.split_at(i),
        None => return home_compressed,
    };

    if components.is_empty() {
        return home_compressed;
    }

    let mut compressed = String::with_capacity(home_compressed.len());

    let mut first_segment = true;
    let mut first_grapheme_in_segment = false;
    for grapheme in components.graphemes(true) {
        if first_segment {
            if grapheme == "~" {
                compressed.push('~');
            } else if grapheme == "/" {
                compressed.push('/');
                first_segment = false;
                first_grapheme_in_segment = true;
            } else {
                compressed.push_str(grapheme);
                first_segment = false;
            }
        } else {
            if grapheme == "/" {
                compressed.push('/');
                first_grapheme_in_segment = true;
            } else if first_grapheme_in_segment && grapheme != "/" {
                compressed.push_str(grapheme);
                first_grapheme_in_segment = grapheme == ".";
            }
        }
    }

    compressed.push_str(last); // last includes the '/'

    compressed
}

fn compress_home_prefix(path: String) -> String {
    let home_path = match home_dir() {
        Some(p) => p.into_string_lossy(),
        None => return path,
    };

    if path.starts_with(&home_path) {
        let mut output = String::with_capacity(path.len() - home_path.len() + 1);
        output.push('~');
        output.push_str(&path[home_path.len()..]);

        return output;
    }

    for (username, user_home) in get_home_dirs() {
        if path.starts_with(&user_home) {
            let mut output =
                String::with_capacity(path.len() - user_home.len() + username.len() + 1);
            output.push('~');
            output.push_str(&username);
            output.push_str(&path[user_home.len()..]);

            return output;
        }
    }

    path
}

fn from_posix<'a>(s: *const c_char) -> &'a OsStr {
    OsStr::from_bytes(unsafe { CStr::from_ptr(s) }.to_bytes())
}

fn get_home_dirs() -> Vec<(String, String)> {
    // (username, home dir)
    let mut users = Vec::new();

    while let Some(entry) = NonNull::new(unsafe { libc::getpwent() }) {
        let entry: &passwd = unsafe { &entry.as_ref() };

        let username = from_posix(entry.pw_name);
        let home = from_posix(entry.pw_dir);
        let shell = from_posix(entry.pw_shell);

        if shell != "/bin/false" && shell != "/sbin/nologin" && home != "/" {
            users.push((
                username.to_string_lossy().into_owned(),
                home.to_string_lossy().into_owned(),
            ));
        }
    }

    unsafe { libc::endpwent() };

    users
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}
