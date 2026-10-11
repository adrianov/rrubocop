//! Naming/BlockForwarding — anonymous `&` vs explicit `&block` (Ruby 3.1+).

use tree_sitter::Node;

use crate::cop::shared::{
    EXCLUDED_BLOCK_PARENTS, METHOD_STOPS, Passes, ScanOpts, node_bytes, replace_arg, scan,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

mod explicit;

pub struct BlockForwarding;

const COP: &str = "Naming/BlockForwarding";

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

fn has_kwarg(params: Node<'_>) -> bool {
    let mut cur = params.walk();
    // RuboCop: kwarg / kwoptarg only (not `**opts`).
    params
        .named_children(&mut cur)
        .any(|n| n.kind() == "keyword_parameter")
}

pub(super) fn scan_method<'a>(
    source: &SourceFile,
    body: Node<'a>,
    use_name: &[u8],
    pass_name: Option<&[u8]>,
    any_inside_block: bool,
    guard_inside_block: bool,
) -> Passes<'a> {
    scan(
        source,
        body,
        use_name,
        pass_name,
        &ScanOpts {
            skip_scopes: &[],
            excluded_parents: EXCLUDED_BLOCK_PARENTS,
            scope_stops: METHOD_STOPS,
            include_lambda: false,
            any_inside_block,
            guard_inside_block,
        },
    )
}

pub(super) fn register(
    cop: &BlockForwarding,
    source: &SourceFile,
    node: Node<'_>,
    style: &str,
    replacement: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(node.start_byte());
    let mut diag = cop.diagnostic(source, line, col, format!("Use {style} block forwarding."));
    diag.highlight_length = node.end_byte().saturating_sub(node.start_byte()).max(1);
    if let Some(text) = replacement
        && replace_arg(source, node, text, corrections, COP)
    {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn check_anonymous(
    cop: &BlockForwarding,
    source: &SourceFile,
    node: Node<'_>,
    params: Node<'_>,
    last: Node<'_>,
    ruby: f64,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(name) = block_param_name(source, last) else {
        return;
    };
    if has_kwarg(params) {
        return;
    }
    let Some(body) = node.child_by_field_name("body") else {
        register(
            cop,
            source,
            last,
            "anonymous",
            Some("&"),
            diagnostics,
            corrections,
        );
        return;
    };
    let found = scan_method(source, body, name, Some(name), true, ruby <= 3.3);
    if found.name_used || found.blocked {
        return;
    }
    for pass in found.nodes {
        register(
            cop,
            source,
            pass,
            "anonymous",
            Some("&"),
            diagnostics,
            corrections,
        );
    }
    register(
        cop,
        source,
        last,
        "anonymous",
        Some("&"),
        diagnostics,
        corrections,
    );
}

impl Cop for BlockForwarding {
    fn name(&self) -> &'static str {
        COP
    }

    fn supports_autocorrect(&self) -> bool {
        true
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
        mut corrections: Option<&mut Vec<Correction>>,
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
            explicit::check_explicit(
                self,
                source,
                node,
                last,
                ruby,
                config.get_str("BlockForwardingName", "block"),
                diagnostics,
                &mut corrections,
            );
        } else {
            check_anonymous(
                self,
                source,
                node,
                params,
                last,
                ruby,
                diagnostics,
                &mut corrections,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::testutil::{
        assert_cop_no_offenses_full_with_config, assert_cop_offenses_full_with_config,
    };

    fn ruby31() -> CopConfig {
        CopConfig {
            options: HashMap::from([(
                "TargetRubyVersion".into(),
                serde_yml::Value::Number(serde_yml::Number::from(3.1)),
            )]),
            ..CopConfig::default()
        }
    }

    fn collect(
        src: &str,
    ) -> (
        crate::parse::source::SourceFile,
        Vec<crate::correction::Correction>,
    ) {
        let source = crate::parse::source::SourceFile::from_bytes("t.rb", src.as_bytes().to_vec());
        let tree = crate::parse::parse_ruby(&source).expect("parse");
        let config = ruby31();
        let mut corrections = Vec::new();
        crate::cop::walker::BatchedWalker::new(vec![&BlockForwarding], vec![&config]).walk(
            &source,
            tree.root_node(),
            &mut Vec::new(),
            Some(&mut corrections),
        );
        (source, corrections)
    }

    fn corrected(src: &str) -> String {
        let (source, corrections) = collect(src);
        String::from_utf8(
            crate::correction::CorrectionSet::from_vec(corrections).apply(source.as_bytes()),
        )
        .expect("utf8")
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
    fn highlight_covers_block_name() {
        let diags = crate::testutil::run_cop_full_with_config(
            &BlockForwarding,
            b"def foo(&block)\n  bar(&block)\nend\n",
            ruby31(),
        );
        assert_eq!(diags.len(), 2);
        assert!(
            diags
                .iter()
                .all(|d| d.correctable && d.highlight_length == 6)
        );
    }

    #[test]
    fn autocorrect_anonymous_block() {
        assert_eq!(
            corrected("def foo(&block)\n  bar(&block)\nend\ndef bare &block\n  bar &block\nend\n"),
            "def foo(&)\n  bar(&)\nend\ndef bare(&)\n  bar(&)\nend\n",
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

    fn ruby_style(version: f64, style: &str) -> CopConfig {
        CopConfig {
            options: HashMap::from([
                (
                    "TargetRubyVersion".into(),
                    serde_yml::Value::Number(serde_yml::Number::from(version)),
                ),
                (
                    "EnforcedStyle".into(),
                    serde_yml::Value::String(style.into()),
                ),
            ]),
            ..CopConfig::default()
        }
    }

    #[test]
    fn inside_block_blocks_the_whole_method_on_ruby33() {
        let src = b"def foo(&)\n  baz { qux(&) }\n  bar(&)\nend\n\
                    def named(&block)\n  baz { qux(&block) }\n  bar(&block)\nend\n";
        assert!(
            crate::testutil::run_cop_full_with_config(
                &BlockForwarding,
                src,
                ruby_style(3.3, "explicit")
            )
            .is_empty()
        );
        assert!(
            crate::testutil::run_cop_full_with_config(
                &BlockForwarding,
                src,
                ruby_style(3.3, "anonymous")
            )
            .is_empty()
        );
    }

    #[test]
    fn explicit_outer_pass_still_registers() {
        let diags = crate::testutil::run_cop_full_with_config(
            &BlockForwarding,
            b"def foo(&)\n  bar(&)\nend\n",
            ruby_style(3.3, "explicit"),
        );
        assert_eq!(diags.len(), 2, "{diags:?}");
    }

    #[test]
    fn ruby34_reports_forward_inside_block() {
        let diags = crate::testutil::run_cop_full_with_config(
            &BlockForwarding,
            b"def foo(&block)\n  baz { bar(&block) }\nend\n",
            ruby_style(3.4, "anonymous"),
        );
        assert_eq!(diags.len(), 2, "{diags:?}");
    }
}
