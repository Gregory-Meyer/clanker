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
