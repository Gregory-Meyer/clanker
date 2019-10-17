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
    collections::HashMap,
    iter::{FromIterator, IntoIterator},
};

use unicode_segmentation::UnicodeSegmentation;

pub struct GraphemeClusterTrie<'a> {
    root: Node<'a>,
}

impl<'a> GraphemeClusterTrie<'a> {
    pub fn shortest_unique_prefix<'b>(&self, s: &'b str) -> Option<&'b str> {
        let mut total_len = 0;

        let mut current = &self.root;

        for grapheme in s.graphemes(true) {
            total_len += grapheme.len();

            if let Some(child) = current.children.get(grapheme) {
                current = child;
            } else {
                return Some(&s[..total_len]);
            }
        }

        None
    }
}

impl<'a> FromIterator<&'a str> for GraphemeClusterTrie<'a> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> GraphemeClusterTrie<'a> {
        let mut root = Node::new();

        for mut s in iter {
            let mut current = &mut root;

            for cluster in s.graphemes(true) {
                current = current.children.entry(cluster).or_insert_with(Node::new);

                s = &s[cluster.len()..];
            }
        }

        GraphemeClusterTrie { root }
    }
}

struct Node<'a> {
    children: HashMap<&'a str, Node<'a>>,
}

impl<'a> Node<'a> {
    fn new() -> Node<'a> {
        Node {
            children: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let trie: GraphemeClusterTrie = ["aa", "ab", "ac"].into_iter().map(|&s| s).collect();

        assert_eq!(trie.shortest_unique_prefix("ad"), Some("ad"));
        assert_eq!(trie.shortest_unique_prefix("b"), Some("b"));
        assert_eq!(trie.shortest_unique_prefix("a"), None);
        assert_eq!(trie.shortest_unique_prefix("aa"), None);
    }

    #[test]
    fn realworld() {
        let trie: GraphemeClusterTrie = [
            "a1",
            "c",
            // "c++",
            "Desktop",
            "Documents",
            "Downloads",
            "fa2019",
            "julia",
            "latex",
            "miniconda3",
            "Music",
            "Pictures",
            "Public",
            "python",
            "repos",
            // "rust",
            "Templates",
            "Videos",
        ]
        .into_iter()
        .map(|&s| s)
        .collect();

        assert_eq!(trie.shortest_unique_prefix("c++"), Some("c+"));
        assert_eq!(trie.shortest_unique_prefix("rust"), Some("ru"));
    }
}
