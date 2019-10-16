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
    ffi::{CString, OsString},
    io,
    path::PathBuf,
};

mod gct;

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

    for (i, component) in components.split('/').enumerate() {
        if i != 0 {
            compressed.push('/');
        }

        let mut iter = component.graphemes(true);

        if let Some(grapheme) = iter.next() {
            compressed.push_str(grapheme);

            if i == 0 && (grapheme == "~" || grapheme == ".") {
                if let Some(next) = iter.next() {
                    compressed.push_str(next);
                }
            } else if grapheme == "." {
                if let Some(next) = iter.next() {
                    compressed.push_str(next);
                }
            }
        }
    }

    // last includes the '/'
    compressed.push_str(last);

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

    let root_prefix = if cfg!(target_os = "macos") {
        "/var/root"
    } else {
        "/root"
    };

    if path.starts_with(root_prefix) {
        let without_prefix = &path[root_prefix.len()..];

        let mut output = String::with_capacity(without_prefix.len() + 5);
        output.push_str("~root");
        output.push_str(without_prefix);

        return output;
    }

    let home_prefix = if cfg!(target_os = "macos") {
        "/Users/"
    } else {
        "/home/"
    };

    if !path.starts_with(home_prefix) {
        return path;
    }

    let without_prefix = &path[home_prefix.len()..];
    let maybe_username = if let Some(i) = without_prefix.find('/') {
        &without_prefix[..i]
    } else {
        without_prefix
    };

    let to_check = CString::new(maybe_username).unwrap();

    if unsafe { libc::getpwnam(to_check.as_ptr()).is_null() } {
        return path;
    }

    let mut output = String::with_capacity(path.len() - home_prefix.len() + 1);
    output.push('~');
    output.push_str(without_prefix);

    output
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_home() {
        let home = home_dir().unwrap().into_string_lossy();

        assert!(compress(home) == "~");
    }

    #[test]
    fn compress_home_long() {
        let mut path = home_dir().unwrap();
        path.push("include/c++/8.2.1/experimental/bits");
        let path = path.into_string_lossy();
        eprintln!("path: {}", path);

        let compressed = compress(path);
        eprintln!("compressed: {}", compressed);

        assert!(compressed == "~/i/c/8/e/bits");
    }

    #[test]
    fn compress_home_config() {
        let mut path = home_dir().unwrap();
        path.push(".config/sway");
        let path = path.into_string_lossy();

        eprintln!("path: {}", path);

        let compressed = compress(path);
        eprintln!("compressed: {}", compressed);

        assert!(compressed == "~/.c/sway");
    }

    #[test]
    fn compress_long() {
        assert!(
            compress("/usr/include/c++/8.2.1/experimental/bits".to_string()) == "/u/i/c/8/e/bits"
        );
    }

    #[test]
    fn compress_root() {
        let root_home = if cfg!(target_os = "macos") {
            "/var/root"
        } else {
            "/root"
        };

        assert!(compress(root_home.to_string()) == "~root");
    }

    #[test]
    fn compress_root_long() {
        let root_home = if cfg!(target_os = "macos") {
            "/var/root"
        } else {
            "/root"
        };
        let path = format!("{}/include/c++/8.2.1/experimental/bits", root_home);
        let compressed = compress(path);

        eprintln!("{}", compressed);

        assert!(compressed == "~r/i/c/8/e/bits");
    }

    #[test]
    fn compress_many_dots() {
        let path = "/usr/local/bin/.config/.foo/.bar/..baz/qux".to_string();
        let compressed = compress(path);

        eprintln!("{}", compressed);

        assert!(compressed == "/u/l/b/.c/.f/.b/../qux");
    }
}
