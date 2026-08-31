//! Performance/TimesMap — (breadth-first tree-sitter port).

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic};
use crate::parse::source::SourceFile;

pub struct TimesMap;


impl Cop for TimesMap {
    fn name(&self) -> &'static str {
        "Performance/TimesMap"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(method) = call_method_name(source, node) else { return; };
        if method != b"map" && method != b"collect" { return; }
        let Some(recv) = call_receiver(node) else { return; };
        if !matches!(recv.kind(), "call" | "command") { return; }
        if call_method_name(source, recv) != Some(b"times") { return; }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source, line, col,
            "Use `Array.new(...)` instead of `n.times.map` / `n.times.collect`.".to_string(),
        ));
    }
}
