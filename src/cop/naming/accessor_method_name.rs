//! Naming/AccessorMethodName — prefer attr_* over get_/set_.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AccessorMethodName;

fn param_count(node: Node<'_>) -> usize {
    let Some(params) = node
        .child_by_field_name("parameters")
        .or_else(|| {
            let mut cur = node.walk();
            node.named_children(&mut cur)
                .find(|n| matches!(n.kind(), "method_parameters" | "parameters"))
        })
    else {
        return 0;
    };
    let mut cur = params.walk();
    params.named_children(&mut cur).count()
}

fn has_single_positional_arg(node: Node<'_>) -> bool {
    let Some(params) = node.child_by_field_name("parameters").or_else(|| {
        let mut cur = node.walk();
        node.named_children(&mut cur)
            .find(|n| matches!(n.kind(), "method_parameters" | "parameters"))
    }) else {
        return false;
    };
    let mut cur = params.walk();
    let kids: Vec<_> = params.named_children(&mut cur).collect();
    kids.len() == 1 && kids[0].kind() == "identifier"
}

impl Cop for AccessorMethodName {
    fn name(&self) -> &'static str {
        "Naming/AccessorMethodName"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = node_bytes(source, name_node);
        let Some(msg) = accessor_offense_msg(name, node) else {
            return;
        };
        let (line, column) = source.offset_to_line_col(name_node.start_byte());
        diagnostics.push(self.diagnostic(source, line, column, msg.into()));
    }
}

fn accessor_offense_msg(name: &[u8], node: Node<'_>) -> Option<&'static str> {
    if name.ends_with(b"!") || name.ends_with(b"?") || name.ends_with(b"=") {
        return None;
    }
    if name.starts_with(b"get_") && param_count(node) == 0 {
        Some("Do not prefix reader method names with `get_`. (https://rubystyle.guide#accessor_methods)")
    } else if name.starts_with(b"set_") && has_single_positional_arg(node) {
        Some("Do not prefix writer method names with `set_`. (https://rubystyle.guide#accessor_methods)")
    } else {
        None
    }
}
