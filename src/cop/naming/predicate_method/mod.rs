//! Naming/PredicateMethod — `?` suffix when all returns are boolean.

mod boolean;
mod returns;

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

use boolean::{boolean_return, is_non_boolean_literal, unknown_call};
use returns::{collect_returns, normalize_values, Ret};

pub struct PredicateMethod;

const OPERATORS: &[&[u8]] = &[
    b"|", b"^", b"&", b"<=>", b"==", b"===", b"=~", b">", b">=", b"<", b"<=", b"<<", b">>", b"+",
    b"-", b"*", b"/", b"%", b"**", b"~", b"+@", b"-@", b"!@", b"~@", b"[]", b"[]=", b"!", b"!=",
    b"!~", b"`",
];

fn allowed_method(name: &[u8], config: &CopConfig) -> bool {
    match config.options.get("AllowedMethods") {
        Some(serde_yml::Value::Sequence(items)) => items
            .iter()
            .any(|v| v.as_str().is_some_and(|s| s.as_bytes() == name)),
        Some(serde_yml::Value::String(s)) => s.as_bytes() == name,
        None => name == b"call",
        _ => false,
    }
}

fn all_boolean(source: &SourceFile, values: &[Ret<'_>], config: &CopConfig) -> bool {
    let filtered: Vec<_> = values
        .iter()
        .copied()
        .filter(|v| !matches!(v, Ret::Node(n) if n.kind() == "super"))
        .collect();
    !filtered.is_empty()
        && filtered.iter().all(|v| match v {
            Ret::Nil => false,
            Ret::Node(n) => boolean_return(source, *n, config),
        })
}

fn potential_non_predicate(source: &SourceFile, values: &[Ret<'_>], config: &CopConfig) -> bool {
    let conservative = config.get_str("Mode", "conservative") != "aggressive";
    if conservative
        && values
            .iter()
            .any(|v| matches!(v, Ret::Node(n) if boolean_return(source, *n, config)))
    {
        return false;
    }
    values.iter().any(|v| match v {
        Ret::Nil => true,
        Ret::Node(n) => is_non_boolean_literal(*n),
    })
}

fn acceptable(source: &SourceFile, values: &[Ret<'_>], config: &CopConfig) -> bool {
    if config.get_str("Mode", "conservative") != "conservative" {
        return false;
    }
    values.iter().any(|v| match v {
        Ret::Nil => false,
        Ret::Node(n) => n.kind() == "super" || unknown_call(source, *n, config),
    })
}

fn skipped_name(name: &[u8], config: &CopConfig) -> bool {
    name == b"initialize"
        || OPERATORS.contains(&name)
        || allowed_method(name, config)
        || (config.get_bool("AllowBangMethods", false) && name.ends_with(b"!"))
}

fn report(
    cop: &PredicateMethod,
    source: &SourceFile,
    name_node: Node<'_>,
    is_pred: bool,
    values: &[Ret<'_>],
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(name_node.start_byte());
    if is_pred && potential_non_predicate(source, values, config) {
        diagnostics.push(cop.diagnostic(
            source,
            line,
            col,
            "Non-predicate method names should not end with `?`.".into(),
        ));
    } else if !is_pred && all_boolean(source, values, config) {
        diagnostics.push(cop.diagnostic(
            source,
            line,
            col,
            "Predicate method names should end with `?`.".into(),
        ));
    }
}

impl Cop for PredicateMethod {
    fn name(&self) -> &'static str {
        "Naming/PredicateMethod"
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
        let Some(name_node) = node.child_by_field_name("name") else {
            return;
        };
        let name = node_bytes(source, name_node);
        if skipped_name(name, config) {
            return;
        }
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        let values = normalize_values(source, collect_returns(body));
        if values.is_empty() || acceptable(source, &values, config) {
            return;
        }
        report(
            self,
            source,
            name_node,
            name.ends_with(b"?"),
            &values,
            config,
            diagnostics,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(PredicateMethod, "cops/naming/predicate_method");
}
