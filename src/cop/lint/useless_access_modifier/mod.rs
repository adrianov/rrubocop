//! Lint/UselessAccessModifier — access modifier with no effect.

mod walk;

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/UselessAccessModifier — access modifier with no effect.
pub struct UselessAccessModifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Vis {
    Public,
    Private,
    Protected,
    ModuleFunction,
}

impl Vis {
    fn from_name(name: &[u8]) -> Option<Self> {
        match name {
            b"public" => Some(Self::Public),
            b"private" => Some(Self::Private),
            b"protected" => Some(Self::Protected),
            b"module_function" => Some(Self::ModuleFunction),
            _ => None,
        }
    }
}

fn is_bare_mod_call(source: &SourceFile, node: Node<'_>) -> bool {
    if node.kind() != "call" || node.child_by_field_name("receiver").is_some() {
        return false;
    }
    if !matches!(
        call_method_name(source, node),
        Some(b"private" | b"protected" | b"public" | b"module_function")
    ) {
        return false;
    }
    match node.child_by_field_name("arguments") {
        None => true,
        Some(args) => {
            let mut cur = args.walk();
            args.named_children(&mut cur).next().is_none()
        }
    }
}

pub(super) fn is_modifier(source: &SourceFile, node: Node<'_>) -> bool {
    (node.kind() == "identifier" && Vis::from_name(node_bytes(source, node)).is_some())
        || is_bare_mod_call(source, node)
}

pub(super) fn modifier_vis(source: &SourceFile, node: Node<'_>) -> Option<Vis> {
    if node.kind() == "identifier" {
        return Vis::from_name(node_bytes(source, node));
    }
    call_method_name(source, node).and_then(Vis::from_name)
}

pub(super) fn report_useless(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    child: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = if child.kind() == "identifier" {
        node_text(source, child)
    } else {
        call_method_name(source, child)
            .map(|m| String::from_utf8_lossy(m).into_owned())
            .unwrap_or_default()
    };
    let (line, col) = source.offset_to_line_col(child.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Useless `{name}` access modifier."),
    ));
}

pub(super) fn apply_modifier<'a>(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    child: Node<'a>,
    new_vis: Vis,
    cur: Vis,
    unused: Option<Node<'a>>,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Vis, Option<Node<'a>>) {
    if new_vis == cur {
        report_useless(cop, source, child, diagnostics);
        return (cur, unused);
    }
    if let Some(prev) = unused {
        report_useless(cop, source, prev, diagnostics);
    }
    (new_vis, Some(child))
}

impl Cop for UselessAccessModifier {
    fn name(&self) -> &'static str {
        "Lint/UselessAccessModifier"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class", "module", "singleton_class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        walk::scan_body(self, source, body, config, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    use std::collections::HashMap;

    crate::cop_fixture_tests!(UselessAccessModifier, "cops/lint/useless_access_modifier");

    #[test]
    fn no_offense_memoize_fixture() {
        crate::testutil::assert_cop_no_offenses_full(
            &UselessAccessModifier,
            include_bytes!(
                "../../../../tests/fixtures/cops/lint/useless_access_modifier/no_offense_memoize.rb"
            ),
        );
    }

    fn opts(pairs: &[(&str, serde_yml::Value)]) -> CopConfig {
        let mut config = CopConfig::default();
        config.options = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.clone()))
            .collect::<HashMap<_, _>>();
        config
    }

    #[test]
    fn class_methods_context_creating_keeps_outer_private() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &UselessAccessModifier,
            b"\
module M\n\
  class_methods do\n\
    def a; end\n\
    private\n\
    def b; end\n\
  end\n\
  private\n\
  def c; end\n\
end\n",
            opts(&[(
                "ContextCreatingMethods",
                serde_yml::Value::Sequence(vec![serde_yml::Value::String(
                    "class_methods".into(),
                )]),
            )]),
        );
    }

    #[test]
    fn included_skipped_when_active_support_extensions_enabled() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &UselessAccessModifier,
            b"\
module M\n\
  included do\n\
    private\n\
    def a; end\n\
  end\n\
  private\n\
  def b; end\n\
end\n",
            opts(&[("ActiveSupportExtensionsEnabled", serde_yml::Value::Bool(true))]),
        );
    }
}
