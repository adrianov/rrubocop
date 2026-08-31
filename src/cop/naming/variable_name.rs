//! Naming/VariableName — locals/ivars snake_case (EnforcedStyle).

use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct VariableName;

// Locals, `@ivar`, `@@cvar`, `$gvar` — RuboCop strips sigils when matching style.
static SNAKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A(?:@@|@|\$)?[\p{Ll}\d_]+[!?=]?\z").unwrap());

impl Cop for VariableName {
    fn name(&self) -> &'static str {
        "Naming/VariableName"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["assignment", "operator_assignment"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if config.get_str("EnforcedStyle", "snake_case") != "snake_case" {
            return;
        }
        let Some(name_node) = node.child_by_field_name("left") else {
            return;
        };
        if !matches!(
            name_node.kind(),
            "identifier" | "instance_variable" | "class_variable"
        ) {
            return;
        }
        let name = node_bytes(source, name_node);
        if name == b"_" || SNAKE.is_match(&String::from_utf8_lossy(name)) {
            return;
        }
        let (line, column) = source.offset_to_line_col(name_node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            "Use snake_case for variable names.".into(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(VariableName, "cops/naming/variable_name");
}
