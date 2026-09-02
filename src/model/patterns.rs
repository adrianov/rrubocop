//! Pattern-match binders (`=>`, `in`, hash/array patterns).

use tree_sitter::Node;

use super::builder::Builder;
use super::{IntroKind, ScopeId, Write, WriteKind};

const PATTERN_KINDS: &[&str] = &[
    "hash_pattern",
    "array_pattern",
    "find_pattern",
    "alt_pattern",
    "as_pattern",
    "keyword_pattern",
];

fn is_pattern_kind(kind: &str) -> bool {
    PATTERN_KINDS.contains(&kind)
}

impl Builder<'_> {
    /// Pattern-related write nodes. Returns true when handled.
    pub(super) fn walk_pattern_write(
        &mut self,
        n: Node,
        kind: &str,
        scope: ScopeId,
        under_defined: bool,
    ) -> bool {
        match kind {
            "in_clause" => {
                if let Some(pat) = n.child_by_field_name("pattern") {
                    self.bind_pattern(pat, scope);
                }
                if let Some(b) = n.child_by_field_name("body") {
                    self.walk(b, scope, under_defined);
                }
                true
            }
            "match_pattern" | "match_pattern_guard" => {
                self.walk_match_pattern(n, scope, under_defined);
                true
            }
            "hash_pattern" | "array_pattern" | "find_pattern" | "alt_pattern" => {
                self.bind_pattern(n, scope);
                true
            }
            "keyword_pattern" => {
                self.bind_keyword_pattern(n, scope);
                true
            }
            _ => false,
        }
    }

    fn walk_match_pattern(&mut self, n: Node, scope: ScopeId, under_defined: bool) {
        // `value => pattern` — value is a read expression; pattern binds locals.
        let mut cur = n.walk();
        for child in n.children(&mut cur) {
            if !child.is_named() {
                continue;
            }
            if is_pattern_kind(child.kind()) {
                self.bind_pattern(child, scope);
            } else if !matches!(
                child.kind(),
                "variable_reference_pattern" | "expression_reference_pattern"
            ) {
                self.walk(child, scope, under_defined);
            }
        }
    }

    fn bind_pattern(&mut self, n: Node, scope: ScopeId) {
        match n.kind() {
            "keyword_pattern" => self.bind_keyword_pattern(n, scope),
            "identifier" => self.bind_pattern_name(n, scope),
            _ => {
                let mut cur = n.walk();
                for child in n.children(&mut cur) {
                    if child.is_named() {
                        self.bind_pattern(child, scope);
                    }
                }
            }
        }
    }

    fn bind_keyword_pattern(&mut self, n: Node, scope: ScopeId) {
        // `key:` binds `key`; `key: name` binds `name`.
        let mut cur = n.walk();
        let idents: Vec<_> = n
            .children(&mut cur)
            .filter(|c| c.kind() == "identifier")
            .collect();
        // Explicit value wins (`key: name`); shorthand falls through to symbol.
        if let Some(explicit) = idents.last() {
            self.bind_pattern_name(*explicit, scope);
            return;
        }
        if let Some(key) = n
            .children(&mut n.walk())
            .find(|c| c.kind() == "hash_key_symbol")
        {
            self.bind_pattern_name(key, scope);
        }
    }

    fn bind_pattern_name(&mut self, n: Node, scope: ScopeId) {
        let name = self.text(n).to_string();
        if name.is_empty() {
            return;
        }
        let w = Write {
            byte: n.start_byte(),
            node_id: n.id(),
            kind: WriteKind::Masgn,
            rhs: None,
        };
        self.record_write(scope, &name, w, IntroKind::Pattern);
    }
}
