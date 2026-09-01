//! FactoryBot/AssociationStyle — prefer implicit or explicit associations.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, call_receiver, method_node};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct AssociationStyle;

const FB_INCLUDE: &[&str] = &[
    "**/*_spec.rb",
    "**/spec/**/*",
    "**/test/**/*",
    "**/features/**/*",
    "**/factories/**/*",
    "**/factory.rb",
];

const DSL: &[&[u8]] = &[
    b"association",
    b"factory",
    b"trait",
    b"transient",
    b"sequence",
    b"after",
    b"before",
];

fn call_block(node: Node<'_>) -> Option<Node<'_>> {
    let mut cur = node.walk();
    node.children(&mut cur)
        .find(|c| matches!(c.kind(), "do_block" | "block"))
}

fn is_explicit_association(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(node.kind(), "call" | "command")
        && call_receiver(node).is_none()
        && call_method_name(source, node) == Some(b"association")
}

fn has_keyword_arg(node: Node<'_>) -> bool {
    let Some(args) = node.child_by_field_name("arguments") else {
        return false;
    };
    let mut cur = args.walk();
    args.named_children(&mut cur)
        .any(|c| matches!(c.kind(), "pair" | "hash" | "bare_hash"))
}

fn report(cop: &AssociationStyle, source: &SourceFile, node: Node<'_>, msg: &str, diagnostics: &mut Vec<Diagnostic>) {
    let meth = method_node(node).unwrap_or(node);
    let (line, col) = source.offset_to_line_col(meth.start_byte());
    diagnostics.push(cop.diagnostic(source, line, col, msg.into()));
}

fn check_implicit(
    cop: &AssociationStyle,
    source: &SourceFile,
    child: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if is_explicit_association(source, child) && !has_keyword_arg(child) {
        report(cop, source, child, "Use implicit style to define associations.", diagnostics);
    }
}

fn name_ok_for_implicit(name: &[u8]) -> bool {
    !DSL.iter().any(|&d| d == name) && !name.iter().any(|&b| b.is_ascii_uppercase())
}

/// RuboCop docs: implicit association = receiverless call with **no arguments**.
/// Tree-sitter: bare `user` is an `identifier`; `user do` / `name { }` attach a block;
/// `email "x"` / `status(:a)` have an argument list — those are attributes, not associations.
fn is_implicit_assoc_candidate(source: &SourceFile, child: Node<'_>) -> bool {
    match child.kind() {
        "identifier" => name_ok_for_implicit(crate::cop::shared::node_bytes(source, child)),
        "call" | "command" => {
            if call_receiver(child).is_some() || call_block(child).is_some() {
                return false;
            }
            if !argument_nodes(child).is_empty() {
                return false;
            }
            call_method_name(source, child).is_some_and(name_ok_for_implicit)
        }
        _ => false,
    }
}

fn check_explicit(
    cop: &AssociationStyle,
    source: &SourceFile,
    child: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if is_implicit_assoc_candidate(source, child) {
        report(cop, source, child, "Use explicit style to define associations.", diagnostics);
    }
}

fn recurse_nested(
    cop: &AssociationStyle,
    source: &SourceFile,
    child: Node<'_>,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !matches!(child.kind(), "call" | "command") {
        return;
    }
    let Some(m) = call_method_name(source, child) else {
        return;
    };
    if m != b"trait" && m != b"factory" {
        return;
    }
    if let Some(body) = call_block(child).and_then(|b| b.child_by_field_name("body")) {
        check_body(cop, source, body, style, false, diagnostics);
    }
}

fn check_body(
    cop: &AssociationStyle,
    source: &SourceFile,
    body: Node<'_>,
    style: &str,
    skip_nested_traits: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut cur = body.walk();
    for child in body.named_children(&mut cur) {
        match style {
            "implicit" => check_implicit(cop, source, child, diagnostics),
            "explicit" => check_explicit(cop, source, child, diagnostics),
            _ => {}
        }
        if !skip_nested_traits {
            recurse_nested(cop, source, child, style, diagnostics);
        }
    }
}

impl Cop for AssociationStyle {
    fn name(&self) -> &'static str {
        "FactoryBot/AssociationStyle"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn default_include(&self) -> &'static [&'static str] {
        FB_INCLUDE
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
        if call_receiver(node).is_some() {
            return;
        }
        let Some(method) = call_method_name(source, node) else {
            return;
        };
        if method != b"factory" && method != b"trait" {
            return;
        }
        let Some(body) = call_block(node).and_then(|b| b.child_by_field_name("body")) else {
            return;
        };
        let style = config.get_str("EnforcedStyle", "implicit");
        check_body(self, source, body, style, method == b"factory", diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(AssociationStyle, "cops/factory_bot/association_style");

    fn explicit_cfg() -> CopConfig {
        let mut c = CopConfig::default();
        c.options.insert(
            "EnforcedStyle".into(),
            serde_yml::Value::String("explicit".into()),
        );
        c
    }

    #[test]
    fn explicit_flags_bare_assoc_not_attrs() {
        let diags = crate::testutil::run_cop_full_with_config(
            &AssociationStyle,
            br#"
factory :asset do
  account
  name { "x" }
  email "a@b.c"
  status(:active)
end
"#,
            explicit_cfg(),
        );
        assert_eq!(diags.len(), 1, "got: {diags:?}");
        assert!(diags[0].message.contains("explicit"));
        assert_eq!(diags[0].location.line, 3);
    }
}
