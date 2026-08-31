//! Style/YAMLFileRead — prefer YAML.load_file.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, is_const_named};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct YAMLFileRead;

impl Cop for YAMLFileRead {
    fn name(&self) -> &'static str {
        "Style/YAMLFileRead"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !is_yaml_file_read(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Use `YAML.load_file` instead of `YAML.load(File.read(...))`.".to_string(),
        ));
    }
}

fn is_yaml_file_read(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(method) = call_method_name(source, node) else {
        return false;
    };
    if method != b"load" && method != b"safe_load" && method != b"parse" {
        return false;
    }
    let Some(recv) = call_receiver(node) else {
        return false;
    };
    if !is_const_named(source, recv, b"YAML") {
        return false;
    }
    let args = argument_nodes(node);
    let Some(arg) = args.first() else {
        return false;
    };
    is_file_read(source, *arg)
}

fn is_file_read(source: &SourceFile, arg: Node<'_>) -> bool {
    if arg.kind() != "call" {
        return false;
    }
    if call_method_name(source, arg) != Some(b"read") {
        return false;
    }
    call_receiver(arg).is_some_and(|r| is_const_named(source, r, b"File"))
}
