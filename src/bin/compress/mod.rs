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
