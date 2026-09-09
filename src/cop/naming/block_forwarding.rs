//! Naming/BlockForwarding — anonymous `&` vs explicit `&block` (Ruby 3.1+).

use tree_sitter::Node;

use crate::cop::shared::{for_each_descendant, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BlockForwarding;

fn last_param(params: Node<'_>) -> Option<Node<'_>> {
    let mut cur = params.walk();
    params.named_children(&mut cur).last()
}

fn block_param_name<'a>(source: &'a SourceFile, param: Node<'_>) -> Option<&'a [u8]> {
    if param.kind() != "block_parameter" {
        return None;
    }
    let mut cur = param.walk();
    param
        .named_children(&mut cur)
        .find(|n| n.kind() == "identifier")
        .map(|n| node_bytes(source, n))
}

fn is_anonymous_block_param(param: Node<'_>) -> bool {
    param.kind() == "block_parameter"
        && {
            let mut cur = param.walk();
            param
                .named_children(&mut cur)
                .all(|n| n.kind() != "identifier")
        }
}

fn has_kwarg(params: Node<'_>) -> bool {
    let mut cur = params.walk();
    // RuboCop: kwarg / kwoptarg only (not `**opts`).
    params
        .named_children(&mut cur)
        .any(|n| n.kind() == "keyword_parameter")
}

fn block_pass_name<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    if node.kind() != "block_argument" {
        return None;
    }
    let mut cur = node.walk();
    let ident = node
        .named_children(&mut cur)
        .find(|n| n.kind() == "identifier")?;
    Some(node_bytes(source, ident))
}

fn inside_block(node: Node<'_>) -> bool {
    let mut p = node.parent();
    while let Some(cur) = p {
        if matches!(cur.kind(), "block" | "do_block") {
            return true;
        }
        if matches!(cur.kind(), "method" | "singleton_method") {
            return false;
        }
        p = cur.parent();
    }
    false
}

fn name_used_as_local(source: &SourceFile, body: Node<'_>, name: &[u8]) -> bool {
    let mut used = false;
    for_each_descendant(body, |n| {
        if used || n.kind() != "identifier" || node_bytes(source, n) != name {
            return;
        }
        if n.parent().is_some_and(|p| p.kind() == "block_argument") {
            return;
        }
        // Skip the parameter definition itself.
        if n.parent().is_some_and(|p| p.kind() == "block_parameter") {
            return;
        }
        used = true;
    });
    used
}

fn collect_matching_passes<'a>(
    source: &SourceFile,
    body: Node<'a>,
    name: &[u8],
    ruby: f64,
) -> Option<Vec<Node<'a>>> {
    let mut out = Vec::new();
    let mut stack = vec![body];
    while let Some(n) = stack.pop() {
        if n.kind() == "block_argument" {
            if ruby <= 3.3 && inside_block(n) {
                return None;
            }
            if block_pass_name(source, n) == Some(name) {
                out.push(n);
            }
        }
        let mut cur = n.walk();
        for child in n.named_children(&mut cur) {
            stack.push(child);
        }
    }
    Some(out)
}

fn register(
    cop: &BlockForwarding,
    source: &SourceFile,
    node: Node<'_>,
    style: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Use {style} block forwarding."),
    ));
}

fn check_anonymous(
    cop: &BlockForwarding,
    source: &SourceFile,
    node: Node<'_>,
    params: Node<'_>,
    last: Node<'_>,
    ruby: f64,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(name) = block_param_name(source, last) else {
        return;
    };
    if has_kwarg(params) {
        return;
    }
    let body = node.child_by_field_name("body");
    if body.is_some_and(|b| name_used_as_local(source, b, name)) {
        return;
    }
    if let Some(body) = body {
        let Some(passes) = collect_matching_passes(source, body, name, ruby) else {
            return;
        };
        for pass in passes {
            register(cop, source, pass, "anonymous", diagnostics);
        }
    }
    register(cop, source, last, "anonymous", diagnostics);
}

fn check_explicit(
    cop: &BlockForwarding,
    source: &SourceFile,
    node: Node<'_>,
    last: Node<'_>,
    ruby: f64,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !is_anonymous_block_param(last) {
        return;
    }
    let Some(body) = node.child_by_field_name("body") else {
        register(cop, source, last, "explicit", diagnostics);
        return;
    };
    if !register_anonymous_passes(cop, source, body, ruby, diagnostics) {
        return;
    }
    register(cop, source, last, "explicit", diagnostics);
}

/// Register offenses on anonymous `&` block passes. Returns false if Ruby ≤3.3 nesting blocks.
fn register_anonymous_passes(
    cop: &BlockForwarding,
    source: &SourceFile,
    body: Node<'_>,
    ruby: f64,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let mut stack = vec![body];
    while let Some(n) = stack.pop() {
        if n.kind() == "block_argument" {
            if ruby <= 3.3 && inside_block(n) {
                return false;
            }
            if block_pass_name(source, n).is_none() && n.named_child_count() == 0 {
                register(cop, source, n, "explicit", diagnostics);
            }
        }
        let mut cur = n.walk();
        for child in n.named_children(&mut cur) {
            stack.push(child);
        }
    }
    true
}

impl Cop for BlockForwarding {
    fn name(&self) -> &'static str {
        "Naming/BlockForwarding"
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
        let ruby = config.get_f64("TargetRubyVersion", 2.7);
        if ruby < 3.1 {
            return;
        }
        let Some(params) = node.child_by_field_name("parameters") else {
            return;
        };
        let Some(last) = last_param(params) else {
            return;
        };
        if config.get_str("EnforcedStyle", "anonymous") == "explicit" {
            check_explicit(self, source, node, last, ruby, diagnostics);
        } else {
            check_anonymous(self, source, node, params, last, ruby, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{
        assert_cop_no_offenses_full_with_config, assert_cop_offenses_full_with_config,
    };
    use std::collections::HashMap;

    fn ruby31() -> CopConfig {
        CopConfig {
            options: HashMap::from([(
                "TargetRubyVersion".into(),
                serde_yml::Value::Number(serde_yml::Number::from(3.1)),
            )]),
            ..CopConfig::default()
        }
    }

    #[test]
    fn offense_fixture() {
        assert_cop_offenses_full_with_config(
            &BlockForwarding,
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/cops/naming/block_forwarding/offense.rb"
            )),
            ruby31(),
        );
    }

    #[test]
    fn no_offense_fixture() {
        assert_cop_no_offenses_full_with_config(
            &BlockForwarding,
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/cops/naming/block_forwarding/no_offense.rb"
            )),
            ruby31(),
        );
    }
}
