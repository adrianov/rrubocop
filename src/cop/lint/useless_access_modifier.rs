use tree_sitter::Node;

use crate::cop::shared::{call_method_name, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/UselessAccessModifier — access modifier with no effect.
pub struct UselessAccessModifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vis {
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

fn is_mod_id(source: &SourceFile, node: Node<'_>) -> bool {
    node.kind() == "identifier" && Vis::from_name(node_bytes(source, node)).is_some()
}

fn is_bare_mod_call(source: &SourceFile, node: Node<'_>) -> bool {
    if node.kind() != "call" {
        return false;
    }
    if !matches!(
        call_method_name(source, node),
        Some(b"private" | b"protected" | b"public" | b"module_function")
    ) {
        return false;
    }
    if node.child_by_field_name("receiver").is_some() {
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

fn is_modifier(source: &SourceFile, node: Node<'_>) -> bool {
    is_mod_id(source, node) || is_bare_mod_call(source, node)
}

fn modifier_vis(source: &SourceFile, node: Node<'_>) -> Option<Vis> {
    if node.kind() == "identifier" {
        return Vis::from_name(node_bytes(source, node));
    }
    call_method_name(source, node).and_then(Vis::from_name)
}

fn is_attr_call(source: &SourceFile, node: Node<'_>) -> bool {
    matches!(
        call_method_name(source, node),
        Some(b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor")
    ) && node.child_by_field_name("receiver").is_none()
}

fn call_wraps_method(node: Node<'_>) -> bool {
    if !matches!(node.kind(), "call" | "command" | "command_call") {
        return false;
    }
    if node.child_by_field_name("receiver").is_some() {
        return false;
    }
    let mut found = false;
    crate::cop::shared::for_each_descendant(node, |n| {
        // Instance `def` only — `def self.x` does not consume access modifiers.
        if n.kind() == "method" {
            found = true;
        }
    });
    found
}

fn is_methodish(source: &SourceFile, node: Node<'_>) -> bool {
    // Access modifiers apply to instance methods / attrs. Singleton defs do
    // not consume a pending modifier (RuboCop skips `defs`).
    node.kind() == "method" || is_attr_call(source, node) || call_wraps_method(node)
}

fn mod_name(source: &SourceFile, node: Node<'_>) -> String {
    if node.kind() == "identifier" {
        return node_text(source, node);
    }
    call_method_name(source, node)
        .map(|m| String::from_utf8_lossy(m).into_owned())
        .unwrap_or_default()
}

fn report_useless(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    child: Node<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = mod_name(source, child);
    let (line, col) = source.offset_to_line_col(child.start_byte());
    diagnostics.push(cop.diagnostic(
        source,
        line,
        col,
        format!("Useless `{name}` access modifier."),
    ));
}

fn scan_modifiers(
    cop: &UselessAccessModifier,
    source: &SourceFile,
    children: &[Node<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut cur = Vis::Public;
    let mut unused: Option<Node<'_>> = None;
    for child in children {
        if let Some(v) = modifier_vis(source, *child).filter(|_| is_modifier(source, *child)) {
            if let Some(prev) = unused.take() {
                report_useless(cop, source, prev, diagnostics);
            }
            if v == cur {
                // Same visibility as current — useless even if methods follow.
                report_useless(cop, source, *child, diagnostics);
            } else {
                unused = Some(*child);
                cur = v;
            }
            continue;
        }
        if is_methodish(source, *child) {
            unused = None;
        }
    }
    if let Some(prev) = unused {
        report_useless(cop, source, prev, diagnostics);
    }
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
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(body) = node.child_by_field_name("body") else {
            return;
        };
        let mut cur = body.walk();
        let children: Vec<_> = body.named_children(&mut cur).collect();
        scan_modifiers(self, source, &children, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(UselessAccessModifier, "cops/lint/useless_access_modifier");

    #[test]
    fn no_offense_memoize_fixture() {
        crate::testutil::assert_cop_no_offenses_full(
            &UselessAccessModifier,
            include_bytes!(
                "../../../tests/fixtures/cops/lint/useless_access_modifier/no_offense_memoize.rb"
            ),
        );
    }
}
