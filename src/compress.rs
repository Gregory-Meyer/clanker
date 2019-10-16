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
    borrow::Borrow,
    cmp, env,
    ffi::{CStr, OsStr, OsString},
    io::{self, ErrorKind},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    ptr::NonNull,
};

mod gct;

use gct::GraphemeClusterTrie;

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

pub fn cwd() -> io::Result<String> {
    current_dir().and_then(|d| compress(&d))
}

fn current_dir() -> io::Result<PathBuf> {
    if let Some(path) = env::var_os("PWD") {
        Ok(PathBuf::from(path))
    } else {
        env::current_dir()
    }
}

fn compress(path: &Path) -> io::Result<String> {
    let (without_prefix, mut buf, mut compressed) = without_prefix(path)?;

    let mut components: Vec<_> = without_prefix.components().collect();

    if let Some(last) = components.pop() {
        'outer: for component in components.into_iter() {
            let component_os = component.as_os_str();
            compressed.push('/');

            let entries = match buf.read_dir() {
                Ok(e) => e,
                Err(_) => {
                    buf.push(component);
                    compressed.push_str(component_os.to_string_lossy().borrow());

                    continue;
                }
            };

            let mut filenames = Vec::new();

            for maybe_entry in entries {
                let entry = match maybe_entry {
                    Ok(e) => e,
                    Err(_) => {
                        buf.push(component);
                        compressed.push_str(component_os.to_string_lossy().borrow());

                        continue 'outer;
                    }
                };

                let filename = entry.file_name();

                if filename == component.as_os_str() {
                    continue;
                }

                filenames.push(filename.into_string_lossy());
            }

            let trie: GraphemeClusterTrie = filenames.iter().map(|s| s.as_str()).collect();
            let component_cow = component_os.to_string_lossy();
            let component_str = component_cow.borrow();

            if let Some(mut prefix) = trie.shortest_unique_prefix(component_str) {
                // avoid compressing ".a" to "." or "..a" to "."/".."
                if prefix.starts_with(".") {
                    const MIN_DISAMBUGABLE_LEN: usize = 3;

                    let search_len = cmp::min(component_str.len(), MIN_DISAMBUGABLE_LEN);

                    if let Some(last_dot_index) = component_str[..search_len].rfind('.') {
                        let ideal_end_index = cmp::min(last_dot_index + 2, component_str.len());

                        let disambugable: &str =
                            &component_str[..cmp::min(ideal_end_index, MIN_DISAMBUGABLE_LEN)];

                        if disambugable.len() > prefix.len() {
                            prefix = disambugable;
                        }
                    } else {
                        prefix =
                            &component_str[..cmp::min(component_str.len(), MIN_DISAMBUGABLE_LEN)];
                    }
                }

                compressed.push_str(prefix);
            } else {
                compressed.push_str(component_str.borrow());
            }

            buf.push(component);
        }

        compressed.push('/');
        compressed.push_str(last.as_os_str().to_string_lossy().borrow());
    }

    Ok(compressed)
}

fn without_prefix(path: &Path) -> io::Result<(&Path, PathBuf, String)> {
    let mut buf: PathBuf = OsString::with_capacity(path.as_os_str().len()).into();
    let mut compressed = String::with_capacity(path.as_os_str().len());

    if let Some(home_dir) = dirs::home_dir() {
        if let Ok(without_prefix) = path.strip_prefix(&home_dir) {
            buf.push(home_dir);
            compressed.push('~');

            return Ok((without_prefix, buf, compressed));
        }
    }

    loop {
        let passwd_ptr = match NonNull::new(unsafe { libc::getpwent() }) {
            Some(p) => p,
            None => break,
        };

        let username = unsafe { CStr::from_ptr(passwd_ptr.as_ref().pw_name) };
        let this_home_dir: &Path = unsafe {
            OsStr::from_bytes(CStr::from_ptr(passwd_ptr.as_ref().pw_dir).to_bytes()).as_ref()
        };

        if let Ok(without_prefix) = path.strip_prefix(this_home_dir) {
            buf.push(this_home_dir);
            compressed.push('~');
            compressed.push_str(username.to_string_lossy().borrow());

            return Ok((without_prefix, buf, compressed));
        }
    }

    path.strip_prefix(OsStr::from_bytes(b"/").as_ref() as &Path)
        .map(move |p| {
            buf.push("/");

            (p, buf, compressed)
        })
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}
