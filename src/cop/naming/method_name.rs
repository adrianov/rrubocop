//! Naming/MethodName — EnforcedStyle snake_case (default).

use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct MethodName;

static SNAKE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A(?:[\p{Ll}\d_]+[!?=]?|[!<>=~+\-*/%&^|]+)\z").unwrap());

impl Cop for MethodName {
    fn name(&self) -> &'static str {
        "Naming/MethodName"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method"]
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
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = node_bytes(source, name_node);
        if SNAKE.is_match(&String::from_utf8_lossy(name)) {
            return;
        }
        let (line, column) = source.offset_to_line_col(name_node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            "Use snake_case for method names. (https://rubystyle.guide#snake-case-symbols-methods-vars)".into(),
        ));
    }
}
