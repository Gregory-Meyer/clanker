// Copyright (C) 2020 Gregory Meyer
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::{
    borrow::Borrow,
    cmp,
    ffi::{OsStr, OsString},
    fs::{self, Metadata},
    io::{self, ErrorKind},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    str,
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

pub fn compress(path: &Path, min_home_dir_uid: u64, max_home_dir_uid: u64) -> io::Result<String> {
    let (without_prefix, mut buf, mut compressed) =
        without_prefix(path, min_home_dir_uid, max_home_dir_uid)?;

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

                if fs::metadata(&entry.path())
                    .as_ref()
                    .map(Metadata::is_dir)
                    .unwrap_or(true)
                {
                    filenames.push(filename.into_string_lossy());
                }
            }

            let trie: GraphemeClusterTrie = filenames.iter().map(|s| s.as_str()).collect();
            let component_cow = component_os.to_string_lossy();
            let component_str = component_cow.borrow();

            if let Some(mut prefix) = trie.shortest_unique_prefix(component_str) {
                // avoid compressing ".a" to "." or "..a" to "."/".."
                if prefix.starts_with('.') {
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
    } else if compressed.is_empty() {
        compressed.push('/');
    }

    Ok(compressed)
}

fn without_prefix(
    path: &Path,
    min_home_dir_uid: u64,
    max_home_dir_uid: u64,
) -> io::Result<(&Path, PathBuf, String)> {
    if let Some(home_dir) = dirs::home_dir() {
        if let Ok(without_prefix) = path.strip_prefix(&home_dir) {
            return Ok((without_prefix, home_dir, "~".to_string()));
        }
    }

    let default_prefixed = || {
        path.strip_prefix(OsStr::from_bytes(b"/").as_ref() as &Path)
            .map(move |p| (p, "/".into(), String::new()))
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
    };

    let passwd_contents = if let Ok(contents) = fs::read("/etc/passwd") {
        contents
    } else {
        return default_prefixed();
    };

    let without_prefix_home_dir_and_username: Vec<_> = passwd_contents
        .split(|&elem| elem == b'\n')
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }

            let mut fields = line.split(|&elem| elem == b':');
            let username = fields.next()?;

            let mut fields = fields.skip(1); // skip password
            let uid: u64 = str::from_utf8(fields.next()?).ok()?.parse().ok()?;

            if uid < min_home_dir_uid || uid > max_home_dir_uid {
                return None;
            }

            let mut fields = fields.skip(2); // skip GID and GECOS
            let home_dir_bytes = fields.next()?;

            if fields.count() != 1 {
                // should be only shell remaining
                return None;
            }

            let home_dir = fs::canonicalize(OsStr::from_bytes(home_dir_bytes)).ok()?;
            let without_prefix = path.strip_prefix(&home_dir).ok()?;

            Some((without_prefix, home_dir, username))
        })
        .collect();

    // remove the longest possible prefix. break ties by lexicographically comparing usernames
    if let Some((without_prefix, home_dir, username_bytes)) = without_prefix_home_dir_and_username
        .into_iter()
        .min_by_key(|&(without_prefix, _, username)| {
            let without_prefix_len = without_prefix.as_os_str().len();

            (without_prefix_len, username)
        })
    {
        let username = String::from_utf8_lossy(username_bytes);

        Ok((without_prefix, home_dir, format!("~{}", username)))
    } else {
        default_prefixed()
    }
}
