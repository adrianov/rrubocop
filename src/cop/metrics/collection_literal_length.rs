//! Metrics/CollectionLiteralLength — array/hash/Set[] too long.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, is_const_named};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CollectionLiteralLength;

impl Cop for CollectionLiteralLength {
    fn name(&self) -> &'static str {
        "Metrics/CollectionLiteralLength"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["array", "hash", "call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let max = config.get_usize("LengthThreshold", 250);
        let Some(count) = collection_len(source, node) else {
            return;
        };
        if count >= max {
            let (line, column) = source.offset_to_line_col(node.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Collection literal is too long. [{count}/{max}]"),
            ));
        }
    }
}

fn collection_len(source: &SourceFile, node: Node<'_>) -> Option<usize> {
    match node.kind() {
        "array" | "hash" => {
            let mut cur = node.walk();
            Some(
                node.named_children(&mut cur)
                    .filter(|n| n.kind() != "," && n.kind() != "comment")
                    .count(),
            )
        }
        "call" => set_bracket_len(source, node),
        _ => None,
    }
}

fn set_bracket_len(source: &SourceFile, node: Node<'_>) -> Option<usize> {
    if call_method_name(source, node) != Some(b"[]") {
        return None;
    }
    let recv = call_receiver(node)?;
    if !is_const_named(source, recv, b"Set") {
        return None;
    }
    Some(argument_nodes(node).len())
}
