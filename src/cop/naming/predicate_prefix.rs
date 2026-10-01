//! Naming/PredicatePrefix — drop forbidden prefixes on predicate methods.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct PredicatePrefix;

const DEFAULT_PREFIXES: &[&str] = &["is_", "has_", "have_", "does_"];

impl Cop for PredicatePrefix {
    fn name(&self) -> &'static str {
        "Naming/PredicatePrefix"
    }

    fn redundant_disable_audit(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method", "call", "command", "command_call"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"define_method", b"define_singleton_method"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some((name, span)) = target(source, node, config) else {
            return;
        };
        report(self, source, name, span, config, diagnostics);
    }
}

fn target<'a>(
    source: &'a SourceFile,
    node: Node<'a>,
    config: &CopConfig,
) -> Option<(&'a str, Node<'a>)> {
    if matches!(node.kind(), "method" | "singleton_method") {
        return def_name(source, node);
    }
    macro_name(source, node, config)
}

fn def_name<'a>(source: &'a SourceFile, node: Node<'a>) -> Option<(&'a str, Node<'a>)> {
    let name_node = node.child_by_field_name("name")?;
    let name = std::str::from_utf8(node_bytes(source, name_node)).ok()?;
    Some((name, name_node))
}

fn macro_name<'a>(
    source: &'a SourceFile,
    node: Node<'a>,
    config: &CopConfig,
) -> Option<(&'a str, Node<'a>)> {
    if call_receiver(node).is_some() || !definition_macro(source, node, config) {
        return None;
    }
    symbol_arg(source, node)
}

fn definition_macro(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    call_method_name(source, node).is_some_and(|method| {
        string_list(
            config,
            "MethodDefinitionMacros",
            &["define_method", "define_singleton_method"],
        )
        .iter()
        .any(|m| m.as_bytes() == method)
    })
}

fn symbol_arg<'a>(source: &'a SourceFile, node: Node<'a>) -> Option<(&'a str, Node<'a>)> {
    let arg = argument_nodes(node).into_iter().next()?;
    Some((plain_symbol(source, arg)?, arg))
}

fn plain_symbol<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a str> {
    if !matches!(node.kind(), "simple_symbol" | "symbol") {
        return None;
    }
    let raw = std::str::from_utf8(node_bytes(source, node)).ok()?;
    let name = raw.strip_prefix(':').unwrap_or(raw);
    (!name.is_empty() && !name.contains(['"', '\''])).then_some(name)
}

fn report(
    cop: &PredicatePrefix,
    source: &SourceFile,
    name: &str,
    span: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let prefixes = string_list(config, "NamePrefix", DEFAULT_PREFIXES);
    let forbidden = string_list(config, "ForbiddenPrefixes", DEFAULT_PREFIXES);
    let allowed = string_list(config, "AllowedMethods", &["is_a?"]);
    for prefix in &prefixes {
        let Some(message) = offense(name, prefix, &forbidden, &allowed) else {
            continue;
        };
        push(cop, source, span, message, diagnostics);
    }
}

fn offense(name: &str, prefix: &str, forbidden: &[String], allowed: &[String]) -> Option<String> {
    if !prefix_applies(name, prefix) || name.ends_with('=') || allowed.iter().any(|a| a == name) {
        return None;
    }
    let expected = expected_name(name, prefix, forbidden);
    (name != expected).then(|| format!("Rename `{name}` to `{expected}`."))
}

fn prefix_applies(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|c| !c.is_ascii_digit())
}

fn expected_name(name: &str, prefix: &str, forbidden: &[String]) -> String {
    let stripped = if forbidden.iter().any(|p| p == prefix) {
        name.replacen(prefix, "", 1)
    } else {
        name.to_string()
    };
    if name.ends_with('?') {
        stripped
    } else {
        format!("{stripped}?")
    }
}

fn push(
    cop: &PredicatePrefix,
    source: &SourceFile,
    span: Node<'_>,
    message: String,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, column) = source.offset_to_line_col(span.start_byte());
    let mut diag = cop.diagnostic(source, line, column, message);
    diag.highlight_length = span.end_byte().saturating_sub(span.start_byte());
    diagnostics.push(diag);
}

fn string_list(config: &CopConfig, key: &str, default: &[&str]) -> Vec<String> {
    match config.options.get(key) {
        Some(serde_yml::Value::Sequence(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        Some(serde_yml::Value::String(s)) => vec![s.clone()],
        _ => default.iter().map(|s| (*s).to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(PredicatePrefix, "cops/naming/predicate_prefix");

    #[test]
    fn prefix_kept_when_not_forbidden() {
        let mut config = CopConfig::default();
        config.options.insert(
            "ForbiddenPrefixes".into(),
            serde_yml::Value::Sequence(vec![]),
        );
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &PredicatePrefix,
            b"def is_even?\nend\n",
            config.clone(),
        );
        let diags = crate::testutil::run_cop_full_with_config(
            &PredicatePrefix,
            b"def is_even\nend\n",
            config,
        );
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "Rename `is_even` to `is_even?`.");
    }
}
