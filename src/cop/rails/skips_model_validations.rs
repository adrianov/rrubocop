//! Rails/SkipsModelValidations — methods that skip Active Record validations.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SkipsModelValidations;

const NEEDS_ARGS: &[&str] = &[
    "decrement!",
    "decrement_counter",
    "increment!",
    "increment_counter",
    "insert",
    "insert!",
    "insert_all",
    "insert_all!",
    "toggle!",
    "update_all",
    "update_attribute",
    "update_column",
    "update_columns",
    "update_counters",
    "upsert",
    "upsert_all",
];

fn in_list(config: &CopConfig, key: &str, name: &str) -> bool {
    config
        .options
        .get(key)
        .and_then(|v| v.as_sequence())
        .is_some_and(|items| items.iter().any(|v| v.as_str() == Some(name)))
}

fn forbidden(config: &CopConfig, name: &str) -> bool {
    if config.options.contains_key("Blacklist") {
        return in_list(config, "Blacklist", name);
    }
    if config.options.contains_key("ForbiddenMethods") {
        return in_list(config, "ForbiddenMethods", name);
    }
    matches!(name, "touch" | "touch_all") || NEEDS_ARGS.contains(&name)
}

fn allowed(config: &CopConfig, name: &str) -> bool {
    if config.options.contains_key("Whitelist") {
        return in_list(config, "Whitelist", name);
    }
    in_list(config, "AllowedMethods", name)
}

fn file_utils(source: &SourceFile, node: Node<'_>) -> bool {
    match node.kind() {
        "constant" => node_bytes(source, node) == b"FileUtils",
        "scope_resolution" => {
            node.child_by_field_name("scope").is_none()
                && node
                    .child_by_field_name("name")
                    .is_some_and(|n| node_bytes(source, n) == b"FileUtils")
        }
        _ => false,
    }
}

fn good_touch(source: &SourceFile, node: Node<'_>, name: &str) -> bool {
    if name != "touch" {
        return false;
    }
    if call_receiver(node).is_some_and(|r| file_utils(source, r)) {
        return true;
    }
    let args = argument_nodes(node);
    args.len() == 1 && matches!(args[0].kind(), "true" | "false")
}

fn sym_key<'a>(source: &'a SourceFile, pair: Node<'_>) -> Option<&'a str> {
    let key = pair.child_by_field_name("key")?;
    if !matches!(
        key.kind(),
        "simple_symbol" | "symbol" | "hash_key_symbol" | "identifier"
    ) {
        return None;
    }
    let text = std::str::from_utf8(node_bytes(source, key)).ok()?;
    Some(text.trim_start_matches(':').trim_end_matches(':'))
}

fn other_symbol(source: &SourceFile, pair: Node<'_>) -> bool {
    !matches!(sym_key(source, pair), None | Some("returning" | "unique_by"))
}

fn hash_pairs<'a>(hash: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = hash.walk();
    hash.named_children(&mut cur)
        .filter(|c| c.kind() == "pair")
        .collect()
}

fn second_arg_ok(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() != "hash" || hash_pairs(node).iter().any(|p| other_symbol(source, *p))
}

fn insert_name(name: &str) -> bool {
    name == "insert" || name == "insert!"
}

fn kw_index(args: &[Node<'_>]) -> usize {
    args.iter()
        .position(|n| n.kind() == "pair")
        .unwrap_or(args.len())
}

fn insert_second_ok(source: &SourceFile, args: &[Node<'_>], kw_at: usize) -> bool {
    if kw_at >= 2 {
        return second_arg_ok(source, args[1]);
    }
    args[kw_at..].iter().any(|p| other_symbol(source, *p))
}

/// A non-hash second argument, or a hash with a real column key, is `String`/`Array#insert`.
fn good_insert(source: &SourceFile, node: Node<'_>, name: &str) -> bool {
    if !insert_name(name) {
        return false;
    }
    let args = argument_nodes(node);
    let kw_at = kw_index(&args);
    if kw_at + usize::from(kw_at < args.len()) < 2 {
        return false;
    }
    insert_second_ok(source, &args, kw_at)
}

fn skip_method(source: &SourceFile, node: Node<'_>, name: &str) -> bool {
    (NEEDS_ARGS.contains(&name) && argument_nodes(node).is_empty())
        || good_touch(source, node, name)
        || good_insert(source, node, name)
}

fn method_name<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a str> {
    std::str::from_utf8(call_method_name(source, node)?).ok()
}

fn push_offense(
    cop: &dyn Cop,
    source: &SourceFile,
    node: Node<'_>,
    name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(
        node.child_by_field_name("method")
            .or_else(|| node.child_by_field_name("name"))
            .unwrap_or(node)
            .start_byte(),
    );
    let mut diag = cop.diagnostic(
        source,
        line,
        col,
        format!("Avoid using `{name}` because it skips validations."),
    );
    diag.highlight_length = name.len();
    diagnostics.push(diag);
}

impl Cop for SkipsModelValidations {
    fn name(&self) -> &'static str {
        "Rails/SkipsModelValidations"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(name) = method_name(source, node) else {
            return;
        };
        if allowed(config, name) || !forbidden(config, name) || skip_method(source, node, name) {
            return;
        }
        push_offense(self, source, node, name, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(SkipsModelValidations, "cops/rails/skips_model_validations");

    fn list_config(key: &str, names: &[&str]) -> CopConfig {
        let mut config = CopConfig::default();
        config.options.insert(
            key.into(),
            serde_yml::Value::Sequence(
                names
                    .iter()
                    .map(|n| serde_yml::Value::String((*n).into()))
                    .collect(),
            ),
        );
        config
    }

    #[test]
    fn allowed_methods_supersede_forbidden() {
        let mut config = list_config("ForbiddenMethods", &["toggle!", "touch"]);
        config.options.insert(
            "AllowedMethods".into(),
            serde_yml::Value::Sequence(vec![serde_yml::Value::String("touch".into())]),
        );
        crate::testutil::assert_cop_offenses_full_with_config(
            &SkipsModelValidations,
            b"user.toggle!(:active)\n     ^^^^^^^ Rails/SkipsModelValidations: Avoid using `toggle!` because it skips validations.\n",
            config.clone(),
        );
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &SkipsModelValidations,
            b"User.touch(:attr)\n",
            config,
        );
    }

    #[test]
    fn blacklist_is_the_forbidden_list() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &SkipsModelValidations,
            b"others.update_all(current: false)\n",
            list_config("Blacklist", &["toggle!"]),
        );
    }
}
