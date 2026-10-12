//! Style/ModuleFunction — `module_function` vs `extend self` in a module.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_method_name, node_bytes, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ModuleFunction;

const USE_MODULE_FUNCTION: &str = "Use `module_function` instead of `extend self`.";
const USE_EXTEND_SELF: &str = "Use `extend self` instead of `module_function`.";
const FORBIDDEN: &str = "Do not use `module_function` or `extend self`.";

impl Cop for ModuleFunction {
    fn name(&self) -> &'static str {
        "Style/ModuleFunction"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn safe_autocorrect(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["module"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if node.kind() != "module" {
            return;
        }
        scan(
            self,
            source,
            config.get_str("EnforcedStyle", "module_function"),
            &body_stmts(node),
            diagnostics,
            &mut corrections,
        );
    }
}

fn scan(
    cop: &ModuleFunction,
    source: &SourceFile,
    style: &str,
    stmts: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if suppressed(style, source, stmts) {
        return;
    }
    for n in stmts.iter().copied() {
        let Some((msg, repl)) = hit(style, source, n) else {
            continue;
        };
        report(cop, source, n, msg, repl, diagnostics, corrections);
    }
}

fn suppressed(style: &str, source: &SourceFile, stmts: &[Node<'_>]) -> bool {
    style == "module_function" && stmts.iter().copied().any(|n| is_private(source, n))
}

fn hit(
    style: &str,
    source: &SourceFile,
    node: Node<'_>,
) -> Option<(&'static str, Option<&'static str>)> {
    let extend = is_extend_self(source, node);
    let module_fn = !extend && is_module_function(source, node);
    match style {
        "module_function" if extend => Some((USE_MODULE_FUNCTION, Some("module_function"))),
        "extend_self" if module_fn => Some((USE_EXTEND_SELF, Some("extend self"))),
        "forbidden" if extend || module_fn => Some((FORBIDDEN, None)),
        _ => None,
    }
}

fn body_stmts<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let Some(body) = node.child_by_field_name("body") else {
        return Vec::new();
    };
    let mut cur = body.walk();
    body.named_children(&mut cur).collect()
}

fn is_extend_self(source: &SourceFile, node: Node<'_>) -> bool {
    if !bare_named(source, node, b"extend") || has_block(node) {
        return false;
    }
    let args = argument_nodes(node);
    args.len() == 1 && args[0].kind() == "self"
}

fn is_module_function(source: &SourceFile, node: Node<'_>) -> bool {
    if node.kind() == "identifier" {
        return node_bytes(source, node) == b"module_function";
    }
    bare_named(source, node, b"module_function") && zero_args(node) && !has_block(node)
}

fn is_private(source: &SourceFile, node: Node<'_>) -> bool {
    if node.kind() == "identifier" {
        return node_bytes(source, node) == b"private";
    }
    bare_named(source, node, b"private") && !has_block(node)
}

fn bare_named(source: &SourceFile, node: Node<'_>, name: &[u8]) -> bool {
    matches!(node.kind(), "call" | "command" | "command_call")
        && node.child_by_field_name("receiver").is_none()
        && call_method_name(source, node) == Some(name)
}

fn zero_args(node: Node<'_>) -> bool {
    node.child_by_field_name("arguments")
        .is_none_or(|args| args.named_child_count() == 0)
}

fn has_block(node: Node<'_>) -> bool {
    node.child_by_field_name("block").is_some()
}

fn report(
    cop: &ModuleFunction,
    source: &SourceFile,
    node: Node<'_>,
    msg: &str,
    replacement: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    let mut diag = cop.diagnostic(source, line, col, msg.to_string());
    diag.highlight_length = node.end_byte().saturating_sub(node.start_byte()).max(1);
    attach(cop, node, replacement, corrections, &mut diag);
    diagnostics.push(diag);
}

fn attach(
    cop: &ModuleFunction,
    node: Node<'_>,
    replacement: Option<&str>,
    corrections: &mut Option<&mut Vec<Correction>>,
    diag: &mut Diagnostic,
) {
    let Some(text) = replacement else {
        diag.correctable = false;
        return;
    };
    if push_replace(
        corrections,
        node.start_byte(),
        node.end_byte(),
        text,
        cop.name(),
    ) {
        diag.corrected = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    use crate::testutil::{
        assert_cop_no_offenses_full_with_config, assert_cop_offenses_full_with_config,
    };

    crate::cop_fixture_tests!(ModuleFunction, "cops/style/module_function");

    fn style_config(style: &str) -> CopConfig {
        let mut config = CopConfig::default();
        config.options.insert(
            "EnforcedStyle".into(),
            serde_yml::Value::String(style.into()),
        );
        config
    }

    fn rewritten(src: &str, style: &str) -> (Vec<Diagnostic>, String) {
        let source = SourceFile::from_bytes("test.rb", src.as_bytes().to_vec());
        let (diagnostics, corrections) = lint_corr(&source, style);
        (diagnostics, apply_corr(&source, corrections))
    }

    fn lint_corr(source: &SourceFile, style: &str) -> (Vec<Diagnostic>, Vec<Correction>) {
        let config = style_config(style);
        let tree = crate::parse::parse_ruby(source).expect("parse");
        let mut diagnostics = Vec::new();
        let mut corrections = Vec::new();
        crate::cop::walker::BatchedWalker::new(vec![&ModuleFunction], vec![&config]).walk(
            source,
            tree.root_node(),
            &mut diagnostics,
            Some(&mut corrections),
        );
        (diagnostics, corrections)
    }

    fn apply_corr(source: &SourceFile, corrections: Vec<Correction>) -> String {
        String::from_utf8(
            crate::correction::CorrectionSet::from_vec(corrections).apply(source.as_bytes()),
        )
        .expect("utf8")
    }

    fn assert_rewrite(src: &str, style: &str, want: &str, len: usize, corrected: bool) {
        let (diags, out) = rewritten(src, style);
        assert_eq!(out, want);
        assert!(diags.iter().all(|d| {
            d.highlight_length == len && d.corrected == corrected && (corrected || !d.correctable)
        }));
    }

    #[test]
    fn extend_self_style_flags_bare_module_function() {
        let cfg = style_config("extend_self");
        assert_cop_offenses_full_with_config(
            &ModuleFunction,
            b"module Test\n  module_function\n  ^^^^^^^^^^^^^^^ Style/ModuleFunction: Use `extend self` instead of `module_function`.\n  def test; end\nend\n",
            cfg.clone(),
        );
        assert_cop_offenses_full_with_config(
            &ModuleFunction,
            b"module TelemetryMetrics\n  EXPORTER = 1\n\n  module_function\n  ^^^^^^^^^^^^^^^ Style/ModuleFunction: Use `extend self` instead of `module_function`.\n\n  def metric_points(name)\n    name\n  end\nend\n",
            cfg.clone(),
        );
        assert_cop_offenses_full_with_config(
            &ModuleFunction,
            b"module Only\n  module_function\n  ^^^^^^^^^^^^^^^ Style/ModuleFunction: Use `extend self` instead of `module_function`.\nend\n",
            cfg.clone(),
        );
        assert_cop_no_offenses_full_with_config(
            &ModuleFunction,
            b"module Test\n  extend self\n  def test; end\nend\n\nmodule Args\n  def test; end\n  module_function :test\nend\n\nmodule Inside\n  def test\n    module_function\n  end\nend\n\nclass Test\n  module_function\n  def test; end\nend\n",
            cfg,
        );
    }

    #[test]
    fn forbidden_style_flags_both_without_correctable() {
        let diags = crate::testutil::run_cop_full_with_config(
            &ModuleFunction,
            b"module Test\n  extend self\n  def test; end\n  private\n  def hidden; end\nend\n\nmodule Other\n  module_function\n  def test; end\nend\n",
            style_config("forbidden"),
        );
        assert_eq!(diags.len(), 2);
        assert!(
            diags
                .iter()
                .all(|d| !d.correctable && d.message == FORBIDDEN)
        );
    }

    #[test]
    fn autocorrect_extend_self() {
        assert_rewrite(
            "module Test\n  extend self\n  def test; end\nend\n",
            "module_function",
            "module Test\n  module_function\n  def test; end\nend\n",
            11,
            true,
        );
    }

    #[test]
    fn autocorrect_extend_self_parens_and_spaces() {
        assert_rewrite(
            "module Test\n  extend(self)\n  def test; end\nend\n",
            "module_function",
            "module Test\n  module_function\n  def test; end\nend\n",
            12,
            true,
        );
        assert_rewrite(
            "module Test\n  extend  self\n  def test; end\nend\n",
            "module_function",
            "module Test\n  module_function\n  def test; end\nend\n",
            12,
            true,
        );
    }

    #[test]
    fn autocorrect_module_function() {
        assert_rewrite(
            "module Test\n  module_function()\n  def test; end\nend\n",
            "extend_self",
            "module Test\n  extend self\n  def test; end\nend\n",
            17,
            true,
        );
        assert_rewrite(
            "module Test\n  module_function\n  def test; end\nend\n",
            "extend_self",
            "module Test\n  extend self\n  def test; end\nend\n",
            15,
            true,
        );
    }

    #[test]
    fn forbidden_style_is_not_correctable() {
        assert_rewrite(
            "module Test\n  module_function\n  def test; end\nend\n",
            "forbidden",
            "module Test\n  module_function\n  def test; end\nend\n",
            15,
            false,
        );
        assert!(!ModuleFunction.safe_autocorrect());
    }
}
