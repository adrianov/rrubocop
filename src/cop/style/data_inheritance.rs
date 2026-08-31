//! Style/DataInheritance — don't inherit from Data.define.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, is_const_named};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct DataInheritance;

impl Cop for DataInheritance {
    fn name(&self) -> &'static str {
        "Style/DataInheritance"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if !inherits_data_define(source, node) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not inherit from `Data.define`. Extend from `Data` and call `.define` instead."
                .to_string(),
        ));
    }
}

fn inherits_data_define(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(superclass) = node.child_by_field_name("superclass") else {
        return false;
    };
    let sc = unwrap_superclass(superclass);
    sc.kind() == "call"
        && call_method_name(source, sc) == Some(b"define")
        && call_receiver(sc).is_some_and(|r| is_const_named(source, r, b"Data"))
}

fn unwrap_superclass(superclass: Node<'_>) -> Node<'_> {
    if superclass.kind() == "superclass" {
        let mut c = superclass.walk();
        superclass.named_children(&mut c).next().unwrap_or(superclass)
    } else {
        superclass
    }
}
