//! RSpec/DescribedClass — prefer `described_class` over the described constant.

use tree_sitter::Node;

use crate::cop::rspec::helpers::{bare_rspec_call, call_block, is_group, RSPEC_INCLUDE};
use crate::cop::shared::{
    call_method_name, for_each_descendant, node_bytes, push_replace,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct DescribedClass;

fn described_const<'a>(source: &'a SourceFile, node: Node<'_>) -> Option<&'a [u8]> {
    let first = crate::cop::shared::argument_nodes(node).into_iter().next()?;
    matches!(first.kind(), "constant" | "scope_resolution")
        .then(|| node_bytes(source, first))
}

fn is_matching_const(source: &SourceFile, node: Node<'_>, want: &[u8]) -> bool {
    matches!(node.kind(), "constant" | "scope_resolution") && node_bytes(source, node) == want
}

fn skip_describe_arg(node: Node<'_>, describe: Node<'_>) -> bool {
    let Some(args) = describe.child_by_field_name("arguments") else {
        return false;
    };
    let mut cur = args.walk();
    args.named_children(&mut cur).any(|c| c.id() == node.id())
}

/// `Transfer` in `Transfer::FOO` — RuboCop skips when OnlyStaticConstants.
fn is_scope_prefix(node: Node<'_>) -> bool {
    node.parent().is_some_and(|p| {
        p.kind() == "scope_resolution"
            && p.child_by_field_name("scope")
                .is_some_and(|s| s.id() == node.id())
    })
}

fn is_nested_describe_arg(source: &SourceFile, node: Node<'_>) -> bool {
    let Some(p) = node.parent() else {
        return false;
    };
    let Some(gp) = p.parent() else {
        return false;
    };
    if !matches!(gp.kind(), "call" | "command") {
        return false;
    }
    call_method_name(source, gp).is_some_and(is_group) && skip_describe_arg(node, gp)
}

fn report_usage(
    cop: &DescribedClass,
    source: &SourceFile,
    n: Node<'_>,
    const_name: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(n.start_byte());
    let msg = format!(
        "Use `described_class` instead of `{}`.",
        String::from_utf8_lossy(const_name)
    );
    let mut diag = cop.diagnostic(source, line, col, msg);
    if push_replace(
        corrections,
        n.start_byte(),
        n.end_byte(),
        "described_class",
        cop.name(),
    ) {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

impl Cop for DescribedClass {
    fn name(&self) -> &'static str {
        "RSpec/DescribedClass"
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_INCLUDE
    }

    fn supports_autocorrect(&self) -> bool {
        true
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
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if config.get_str("EnforcedStyle", "described_class") != "described_class" {
            return;
        }
        let only_static = config.get_bool("OnlyStaticConstants", true);
        let Some(method) = bare_rspec_call(source, node) else {
            return;
        };
        if !matches!(method, b"describe" | b"xdescribe" | b"fdescribe") {
            return;
        }
        let Some(const_name) = described_const(source, node) else {
            return;
        };
        let Some(block) = call_block(node) else {
            return;
        };
        for_each_descendant(block, |n| {
            if !is_matching_const(source, n, const_name) {
                return;
            }
            if skip_describe_arg(n, node) {
                return;
            }
            if only_static && is_scope_prefix(n) {
                return;
            }
            if is_nested_describe_arg(source, n) {
                return;
            }
            report_usage(self, source, n, const_name, diagnostics, &mut corrections);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(DescribedClass, "cops/rspec/described_class");
}
