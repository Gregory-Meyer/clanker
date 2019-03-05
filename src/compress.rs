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

use std::{env, iter, path::PathBuf};

use unicode_segmentation::UnicodeSegmentation;

pub fn compressed_cwd() -> String {
    compress(cwd())
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
    let components: Vec<&str> = path.split('/').collect();
    let (last, rest) = components.split_last().unwrap();

    if rest.is_empty() {
        return last.to_string();
    }

    let parts: Vec<&str> = rest
        .iter()
        .map(|s| {
            let mut graphemes = s.grapheme_indices(true);
            match graphemes.next() {
                Some((_, g)) => {
                    if g == "." {
                        graphemes
                            .next()
                            .map(|(j, h)| &s[..j + h.len()])
                            .unwrap_or(g)
                    } else {
                        g
                    }
                }
                None => "",
            }
        })
        .chain(iter::once(*last))
        .collect();

    parts.join("/")
}
