//! Layout/DotPosition.

use tree_sitter::Node;

use crate::cop::shared;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct DotPosition;

fn find_dot(bytes: &[u8], recv: Node<'_>, method: Node<'_>) -> Option<usize> {
    let search = &bytes[recv.end_byte()..method.start_byte()];
    let rel = search.iter().position(|&b| b == b'.')?;
    Some(recv.end_byte() + rel)
}

fn fix_leading(cop: &dyn Cop, dot: usize, method: Node<'_>, corr: &mut Vec<Correction>) {
    corr.push(Correction {
        start: dot, end: method.start_byte(), replacement: String::new(),
        cop_name: cop.name(), cop_index: 0,
    });
    corr.push(Correction {
        start: method.start_byte(), end: method.start_byte(), replacement: ".".into(),
        cop_name: cop.name(), cop_index: 1,
    });
}

fn report_leading(
    cop: &dyn Cop, source: &SourceFile, dot: usize, method: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>, corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (dot_line, dot_col) = source.offset_to_line_col(dot);
    let mut diag = cop.diagnostic(
        source, dot_line, dot_col,
        "Place the . on the next line, together with the method name.".into(),
    );
    if let Some(corr) = corrections {
        fix_leading(cop, dot, method, corr);
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn report_trailing(cop: &dyn Cop, source: &SourceFile, dot: usize, diagnostics: &mut Vec<Diagnostic>) {
    let (dot_line, dot_col) = source.offset_to_line_col(dot);
    diagnostics.push(cop.diagnostic(
        source, dot_line, dot_col,
        "Place the . on the previous line, together with the method receiver.".into(),
    ));
}

impl Cop for DotPosition {
    fn name(&self) -> &'static str { "Layout/DotPosition" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["call"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let want_leading = config.get_str("EnforcedStyle", "leading") != "trailing";
        let Some(recv) = node.child_by_field_name("receiver") else { return; };
        let Some(method) = node.child_by_field_name("method") else { return; };
        if shared::node_line(source, recv) == shared::node_line(source, method) { return; }
        let Some(dot) = find_dot(source.as_bytes(), recv, method) else { return; };
        let leading = source.offset_to_line_col(dot).0 == shared::node_line(source, method);
        if want_leading && !leading {
            report_leading(self, source, dot, method, diagnostics, &mut corrections);
        } else if !want_leading && leading {
            report_trailing(self, source, dot, diagnostics);
        }
    }
}
