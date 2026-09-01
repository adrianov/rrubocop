//! Rails/DelegateAllowBlank — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, node_text, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct DelegateAllowBlank;

fn allow_nil_repl(text: &str) -> &'static str {
    if text.ends_with(':') {
        "allow_nil:"
    } else if text.starts_with(':') {
        ":allow_nil"
    } else {
        "allow_nil"
    }
}

impl Cop for DelegateAllowBlank {
    fn name(&self) -> &'static str {
        "Rails/DelegateAllowBlank"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"delegate"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if call_method_name(source, node) != Some(b"delegate") {
            return;
        }
        let Some(key) = find_allow_blank_key(source, node) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(key.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            "`allow_blank` is not a valid option for `delegate`. Did you mean `allow_nil`?"
                .into(),
        );
        if push_replace(
            &mut corrections,
            key.start_byte(),
            key.end_byte(),
            allow_nil_repl(&node_text(source, key)),
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}

fn find_allow_blank_key<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    for arg in argument_nodes(node) {
        if let Some(k) = pair_allow_blank_key(source, arg) {
            return Some(k);
        }
        if arg.kind() == "hash" {
            let mut cur = arg.walk();
            for child in arg.named_children(&mut cur) {
                if let Some(k) = pair_allow_blank_key(source, child) {
                    return Some(k);
                }
            }
        }
    }
    None
}

fn pair_allow_blank_key<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    if node.kind() != "pair" {
        return None;
    }
    let key = node.child_by_field_name("key")?;
    let t = node_text(source, key);
    let name = t.trim().trim_start_matches(':').trim_end_matches(':');
    (name == "allow_blank").then_some(key)
}
