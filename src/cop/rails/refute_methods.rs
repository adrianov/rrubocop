//! Rails/RefuteMethods — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, method_node, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RefuteMethods;

const CORRECTIONS: &[(&[u8], &str)] = &[
    (b"refute", "assert_not"),
    (b"refute_empty", "assert_not_empty"),
    (b"refute_equal", "assert_not_equal"),
    (b"refute_in_delta", "assert_not_in_delta"),
    (b"refute_in_epsilon", "assert_not_in_epsilon"),
    (b"refute_includes", "assert_not_includes"),
    (b"refute_instance_of", "assert_not_instance_of"),
    (b"refute_kind_of", "assert_not_kind_of"),
    (b"refute_nil", "assert_not_nil"),
    (b"refute_operator", "assert_not_operator"),
    (b"refute_predicate", "assert_not_predicate"),
    (b"refute_respond_to", "assert_not_respond_to"),
    (b"refute_same", "assert_not_same"),
    (b"refute_match", "assert_no_match"),
];

fn style_pair<'a>(style: &str, method: &'a [u8]) -> Option<(&'a str, &'a str)> {
    let method_str = std::str::from_utf8(method).unwrap_or("");
    if style == "assert_not" {
        let (_, good) = CORRECTIONS.iter().find(|(b, _)| *b == method)?;
        Some((method_str, *good))
    } else {
        let (bad, _) = CORRECTIONS.iter().find(|(_, g)| g.as_bytes() == method)?;
        Some((method_str, std::str::from_utf8(bad).unwrap_or("")))
    }
}

impl Cop for RefuteMethods {
    fn name(&self) -> &'static str {
        "Rails/RefuteMethods"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/test/**/*"]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        let style = config.get_str("EnforcedStyle", "assert_not");
        let Some((bad, good)) = style_pair(style, method) else {
            return;
        };
        let meth = method_node(node).unwrap_or(node);
        let (line, col) = source.offset_to_line_col(meth.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            format!("Prefer `{good}` over `{bad}`."),
        );
        if push_replace(
            &mut corrections,
            meth.start_byte(),
            meth.end_byte(),
            good,
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
