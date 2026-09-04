//! Layout/HashAlignment.

use tree_sitter::Node;

use crate::cop::layout::align_items;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct HashAlignment;

const MSG: &str = "Align the keys of a hash literal if they span more than one line.";

fn last_arg_style(config: &CopConfig) -> &str {
    config.get_str("EnforcedLastArgumentHashStyle", "always_inspect")
}

/// True when `node` is the last non-comment argument of a call/command.
fn is_last_call_arg(node: Node<'_>) -> bool {
    let Some(args) = node.parent() else {
        return false;
    };
    if !matches!(args.kind(), "argument_list" | "command_argument_list") {
        return false;
    }
    if !args
        .parent()
        .is_some_and(|p| matches!(p.kind(), "call" | "command" | "command_call"))
    {
        return false;
    }
    let mut cur = args.walk();
    args.named_children(&mut cur)
        .filter(|n| n.kind() != "comment")
        .last()
        .is_some_and(|n| n.id() == node.id())
}

fn hash_braced(source: &SourceFile, hash: Node<'_>) -> bool {
    source.as_bytes().get(hash.start_byte()) == Some(&b'{')
}

/// RuboCop `ignore_hash_argument?` for an explicit/`hash` last argument.
fn skip_last_hash(source: &SourceFile, hash: Node<'_>, style: &str) -> bool {
    if !is_last_call_arg(hash) {
        return false;
    }
    match style {
        "always_ignore" => true,
        "ignore_explicit" => hash_braced(source, hash),
        "ignore_implicit" => !hash_braced(source, hash),
        _ => false,
    }
}

fn skip_kwargs(style: &str) -> bool {
    matches!(style, "always_ignore" | "ignore_implicit")
}

/// Alignment layout: prior `EnforcedStyle`, else RuboCop `EnforcedColonStyle`.
fn hash_align_style(config: &CopConfig) -> &str {
    if config.options.contains_key("EnforcedStyle") {
        config.get_str("EnforcedStyle", "key")
    } else {
        config.get_str("EnforcedColonStyle", "key")
    }
}

impl Cop for HashAlignment {
    fn name(&self) -> &'static str {
        "Layout/HashAlignment"
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["hash", "argument_list", "command_argument_list"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let last_arg = last_arg_style(config);
        match node.kind() {
            "argument_list" | "command_argument_list" if skip_kwargs(last_arg) => return,
            "hash" if skip_last_hash(source, node, last_arg) => return,
            _ => {}
        }
        align_items::check_hash_align(
            self,
            source,
            node,
            hash_align_style(config),
            config.get_usize("IndentationWidth", 2),
            MSG,
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    use crate::testutil::{
        assert_cop_no_offenses_full_with_config, assert_cop_offenses_full_with_config,
    };

    crate::cop_fixture_tests!(HashAlignment, "cops/layout/hash_alignment");

    fn style_config(style: &str) -> CopConfig {
        let mut config = CopConfig::default();
        config.options.insert(
            "EnforcedLastArgumentHashStyle".into(),
            serde_yml::Value::String(style.into()),
        );
        config
    }

    #[test]
    fn always_ignore_skips_last_arg_hash_and_kwargs() {
        let cfg = style_config("always_ignore");
        assert_cop_no_offenses_full_with_config(
            &HashAlignment,
            b"foo({\n  a: 1,\n    b: 2\n})\n",
            cfg.clone(),
        );
        assert_cop_no_offenses_full_with_config(
            &HashAlignment,
            b"foo(x,\n  a: 1,\n    b: 2)\n",
            cfg,
        );
    }

    #[test]
    fn always_ignore_still_checks_standalone_hash() {
        assert_cop_offenses_full_with_config(
            &HashAlignment,
            b"{\n  a: 1,\n    b: 2\n    ^^ Layout/HashAlignment: Align the keys of a hash literal if they span more than one line.\n}\n",
            style_config("always_ignore"),
        );
    }

    #[test]
    fn ignore_explicit_skips_braced_last_arg_keeps_kwargs() {
        assert_cop_no_offenses_full_with_config(
            &HashAlignment,
            b"foo({\n  a: 1,\n    b: 2\n})\n",
            style_config("ignore_explicit"),
        );
        assert_cop_offenses_full_with_config(
            &HashAlignment,
            b"foo(x,\n  a: 1,\n    b: 2\n    ^^ Layout/HashAlignment: Align the keys of a hash literal if they span more than one line.\n)\n",
            style_config("ignore_explicit"),
        );
    }

    #[test]
    fn ignore_implicit_skips_kwargs_keeps_braced_last_arg() {
        assert_cop_no_offenses_full_with_config(
            &HashAlignment,
            b"foo(x,\n  a: 1,\n    b: 2)\n",
            style_config("ignore_implicit"),
        );
        assert_cop_offenses_full_with_config(
            &HashAlignment,
            b"foo({\n  a: 1,\n    b: 2\n    ^^ Layout/HashAlignment: Align the keys of a hash literal if they span more than one line.\n})\n",
            style_config("ignore_implicit"),
        );
    }
}
