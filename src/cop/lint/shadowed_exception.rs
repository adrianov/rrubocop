use tree_sitter::Node;

use crate::cop::shared::node_text;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/ShadowedException — broader exception before narrower in rescue chain.
pub struct ShadowedException;

fn exception_names(source: &SourceFile, rescue: Node<'_>) -> Vec<String> {
    let Some(ex) = rescue.child_by_field_name("exceptions") else {
        return vec!["StandardError".into()];
    };
    let mut cur = ex.walk();
    ex.named_children(&mut cur)
        .map(|n| node_text(source, n))
        .collect()
}

fn is_ancestor(child: &str, parent: &str) -> bool {
    const HIER: &[(&str, &str)] = &[
        ("StandardError", "Exception"),
        ("RuntimeError", "StandardError"),
        ("ArgumentError", "StandardError"),
        ("NoMethodError", "NameError"),
        ("NameError", "StandardError"),
        ("TypeError", "StandardError"),
        ("RangeError", "StandardError"),
        ("FloatDomainError", "RangeError"),
        ("IOError", "StandardError"),
        ("SystemCallError", "StandardError"),
        ("Errno", "SystemCallError"),
        ("SignalException", "Exception"),
        ("Interrupt", "SignalException"),
        ("SystemExit", "Exception"),
        ("NoMemoryError", "Exception"),
        ("ScriptError", "Exception"),
        ("SyntaxError", "ScriptError"),
        ("LoadError", "ScriptError"),
        ("NotImplementedError", "ScriptError"),
        ("SecurityError", "Exception"),
        ("EncodingError", "StandardError"),
    ];
    if child == parent {
        return true;
    }
    let mut cur = child;
    for _ in 0..8 {
        let Some((_, p)) = HIER.iter().find(|(c, _)| *c == cur) else {
            return false;
        };
        if *p == parent {
            return true;
        }
        cur = p;
    }
    false
}

fn is_shadowed(name: &str, previous: &[String]) -> bool {
    previous
        .iter()
        .any(|prev| is_ancestor(name, prev) || prev == name)
}

fn find_shadow<'a>(
    source: &SourceFile,
    rescues: &[Node<'a>],
) -> Option<Node<'a>> {
    let mut previous: Vec<String> = Vec::new();
    for &rescue in rescues {
        let names = exception_names(source, rescue);
        if names.iter().any(|n| is_shadowed(n, &previous)) {
            return Some(rescue);
        }
        previous.extend(names);
    }
    None
}

impl Cop for ShadowedException {
    fn name(&self) -> &'static str {
        "Lint/ShadowedException"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["begin"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut cur = node.walk();
        let rescues: Vec<_> = node
            .named_children(&mut cur)
            .filter(|n| n.kind() == "rescue")
            .collect();
        if rescues.len() < 2 {
            return;
        }
        let Some(rescue) = find_shadow(source, &rescues) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(rescue.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not shadow rescued Exceptions.".to_string(),
        ));
    }
}
