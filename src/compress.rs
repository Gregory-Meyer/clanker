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
    ffi::{CString, OsStr, OsString},
    iter,
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
};

use unicode_segmentation::UnicodeSegmentation;

pub fn compressed_cwd() -> String {
    compress(cwd())
}

fn cwd() -> String {
    let cwd = match env::current_dir() {
        Ok(p) => p,
        Err(_) => return "?".to_string(),
    };

    if let Some(home) = dirs::home_dir() {
        if home == cwd {
            return "~".to_string();
        }

        if let Ok(path) = cwd.strip_prefix(&home) {
            return format!("{}", PathBuf::from("~").join(path).display());
        }
    }

    if let Some(p) = compact_user_prefix(&cwd) {
        return p;
    }

    format!("{}", cwd.display())
}

fn compress(path: String) -> String {
    let components: Vec<&str> = path.split('/').collect();
    let (last, rest) = components.split_last().unwrap();

    if rest.is_empty() {
        return last.to_string();
    }

    let parts: Vec<&str> = rest
        .iter()
        .enumerate()
        .map(|(i, c)| trim_component(i, c))
        .chain(iter::once(*last))
        .collect();

    parts.join("/")
}

fn trim_component(index: usize, component: &str) -> &str {
    let mut graphemes = component.grapheme_indices(true);

    match graphemes.next() {
        Some((_, g)) => {
            if g == "." || (index == 0 && g == "~") {
                // or case for when path looks like ~user/some/other/folders
                // /~user/some/other/folders would have ~user as its 2nd component, not first
                graphemes
                    .next()
                    .map(|(j, h)| &component[..j + h.len()])
                    .unwrap_or(g)
            } else {
                g
            }
        }
        None => "",
    }
}

fn compact_user_prefix(path: &Path) -> Option<String> {
    let postfix = match path.strip_prefix(home_dir_prefix()) {
        Ok(p) => p,
        Err(_) => return None,
    };

    if let Some(Component::Normal(username)) = postfix.components().next() {
        if !is_user(username) {
            return None;
        }

        let mut prefix = OsString::from("~");
        prefix.push(username);
        let prefix = prefix.to_string_lossy();

        let without_username = postfix.strip_prefix(username).unwrap();

        if without_username.components().next().is_some() {
            return Some(format!("{}/{}", prefix, without_username.display()));
        } else {
            return Some(prefix.into_owned());
        }
    }

    None
}

fn home_dir_prefix() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/Users/")
    } else {
        PathBuf::from("/home/")
    }
}

fn is_user(maybe_user: &OsStr) -> bool {
    let maybe_user_nullterm = CString::new(maybe_user.as_bytes()).unwrap();

    !unsafe { libc::getpwnam(maybe_user_nullterm.as_ptr()) }.is_null()
}
