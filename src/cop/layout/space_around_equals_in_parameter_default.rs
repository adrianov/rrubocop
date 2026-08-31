//! Layout/SpaceAroundEqualsInParameterDefault.

use tree_sitter::Node;

use crate::cop::layout::report;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SpaceAroundEqualsInParameterDefault;

fn only_hspace(slice: &[u8]) -> bool {
    slice.iter().all(|&b| b == b' ' || b == b'\t')
}

fn has_hspace(slice: &[u8]) -> bool {
    slice.iter().any(|&b| b == b' ' || b == b'\t')
}

fn spacing_ok(want: bool, before: &[u8], after: &[u8]) -> bool {
    if want {
        has_hspace(before) && has_hspace(after) && only_hspace(before) && only_hspace(after)
    } else {
        !has_hspace(before) && !has_hspace(after)
    }
}

fn eq_span(bytes: &[u8], name_end: usize, value_start: usize) -> Option<(usize, &[u8], &[u8])> {
    let rel = bytes[name_end..value_start].iter().position(|&b| b == b'=')?;
    let eq = name_end + rel;
    Some((eq, &bytes[name_end..eq], &bytes[eq + 1..value_start]))
}

impl Cop for SpaceAroundEqualsInParameterDefault {
    fn name(&self) -> &'static str { "Layout/SpaceAroundEqualsInParameterDefault" }
    fn supports_autocorrect(&self) -> bool { true }
    fn interested_node_kinds(&self) -> &'static [&'static str] { &["optional_parameter"] }

    fn check_node(
        &self, source: &SourceFile, node: Node<'_>, config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>, mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let want = config.get_str("EnforcedStyle", "space") != "no_space";
        let bytes = source.as_bytes();
        let Some(name) = node.child_by_field_name("name") else { return; };
        let Some(value) = node.child_by_field_name("value") else { return; };
        let Some((eq, before, after)) = eq_span(bytes, name.end_byte(), value.start_byte()) else { return; };
        if spacing_ok(want, before, after) { return; }
        let ty = if want { "missing" } else { "detected" };
        report::report_fix(
            self, source, eq,
            format!("Surrounding space {ty} in default value assignment."),
            diagnostics, &mut corrections,
            name.end_byte(), value.start_byte(),
            if want { " = ".into() } else { "=".into() },
        );
    }
}
