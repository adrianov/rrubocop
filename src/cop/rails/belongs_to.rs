//! Rails/BelongsTo — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, node_text, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BelongsTo;

fn optional_for_required(val: Node<'_>) -> Option<(&'static str, &'static str)> {
    match val.kind() {
        "false" => Some((
            "true",
            "You specified `required: false`, in Rails > 5.0 the required option is deprecated and you want to use `optional: true`.",
        )),
        "true" => Some((
            "false",
            "You specified `required: true`, in Rails > 5.0 the required option is deprecated and you want to use `optional: false`. Also, consider removing as Rails > 5.0 models associations are required by default.",
        )),
        _ => None,
    }
}

fn required_offense<'a>(
    source: &SourceFile,
    node: Node<'a>,
) -> Option<(Node<'a>, &'static str, &'static str)> {
    if call_method_name(source, node) != Some(b"belongs_to") {
        return None;
    }
    let pair = find_required_pair(source, node)?;
    let val = pair.child_by_field_name("value")?;
    let (optional, msg) = optional_for_required(val)?;
    Some((pair, optional, msg))
}

impl Cop for BelongsTo {
    fn name(&self) -> &'static str {
        "Rails/BelongsTo"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"belongs_to"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some((pair, optional, msg)) = required_offense(source, node) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(pair.start_byte());
        let mut diag = self.diagnostic(source, line, col, msg.to_string());
        if push_replace(
            &mut corrections,
            pair.start_byte(),
            pair.end_byte(),
            format!("optional: {optional}"),
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}

fn find_required_pair<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    for arg in argument_nodes(node) {
        if let Some(p) = pair_if_required(source, arg) {
            return Some(p);
        }
        if arg.kind() == "hash" {
            let mut cur = arg.walk();
            for child in arg.named_children(&mut cur) {
                if let Some(p) = pair_if_required(source, child) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn pair_if_required<'a>(source: &SourceFile, node: Node<'a>) -> Option<Node<'a>> {
    if node.kind() != "pair" {
        return None;
    }
    let key = node.child_by_field_name("key")?;
    let t = node_text(source, key);
    let name = t.trim().trim_start_matches(':').trim_end_matches(':');
    (name == "required").then_some(node)
}
