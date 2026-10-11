//! Explicit `EnforcedStyle`: name an anonymous `&` (`&block` by default).

use tree_sitter::Node;

use super::{BlockForwarding, register, scan_method};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub(super) fn check_explicit(
    cop: &BlockForwarding,
    source: &SourceFile,
    node: Node<'_>,
    last: Node<'_>,
    ruby: f64,
    fwd_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !is_anonymous_block_param(last) {
        return;
    }
    let Some(body) = node.child_by_field_name("body") else {
        register(
            cop,
            source,
            last,
            "explicit",
            Some(&format!("&{fwd_name}")),
            diagnostics,
            corrections,
        );
        return;
    };
    let found = scan_method(source, body, fwd_name.as_bytes(), None, false, ruby <= 3.3);
    if found.blocked {
        return;
    }
    let replacement = (!found.name_used).then(|| format!("&{fwd_name}"));
    for pass in found.nodes {
        register(
            cop,
            source,
            pass,
            "explicit",
            replacement.as_deref(),
            diagnostics,
            corrections,
        );
    }
    register(
        cop,
        source,
        last,
        "explicit",
        replacement.as_deref(),
        diagnostics,
        corrections,
    );
}

fn is_anonymous_block_param(param: Node<'_>) -> bool {
    param.kind() == "block_parameter" && {
        let mut cur = param.walk();
        param
            .named_children(&mut cur)
            .all(|child| child.kind() != "identifier")
    }
}
