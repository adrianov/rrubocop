//! Rails/ReadWriteAttribute — breadth-first tree-sitter port.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, node_text, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ReadWriteAttribute;

fn attribute_rewrite<'a>(
    source: &'a SourceFile,
    node: Node<'_>,
) -> Option<(String, &'a str)> {
    let method = call_method_name(source, node)?;
    let args = argument_nodes(node);
    let repl = match method {
        b"read_attribute" if args.len() == 1 => {
            format!("self[{}]", node_text(source, args[0]))
        }
        b"write_attribute" if args.len() == 2 => {
            format!(
                "self[{}] = {}",
                node_text(source, args[0]),
                node_text(source, args[1])
            )
        }
        _ => return None,
    };
    Some((repl, std::str::from_utf8(method).unwrap_or("")))
}

impl Cop for ReadWriteAttribute {
    fn name(&self) -> &'static str {
        "Rails/ReadWriteAttribute"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/app/models/**/*.rb"]
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
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let Some((repl, method)) = attribute_rewrite(source, node) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(node.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            format!("Prefer `{repl}` over `{method}`."),
        );
        if push_replace(
            &mut corrections,
            node.start_byte(),
            node.end_byte(),
            repl,
            self.name(),
        ) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}
