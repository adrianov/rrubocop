//! Walk tree-sitter AST and dispatch to cops by interested node kinds.

use std::collections::HashMap;

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BatchedWalker<'a> {
    cops: Vec<&'a dyn Cop>,
    configs: Vec<&'a CopConfig>,
    kinds: HashMap<&'static str, Vec<usize>>,
    call_all: Vec<usize>,
}

impl<'a> BatchedWalker<'a> {
    pub fn new(cops: Vec<&'a dyn Cop>, configs: Vec<&'a CopConfig>) -> Self {
        let mut kinds: HashMap<&'static str, Vec<usize>> = HashMap::new();
        let mut call_all = Vec::new();
        for (i, cop) in cops.iter().enumerate() {
            let interested = cop.interested_node_kinds();
            if interested.is_empty() {
                call_all.push(i);
            } else {
                for &k in interested {
                    kinds.entry(k).or_default().push(i);
                }
            }
        }
        Self {
            cops,
            configs,
            kinds,
            call_all,
        }
    }

    pub fn walk(
        &self,
        source: &SourceFile,
        root: Node<'_>,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        self.visit(source, root, diagnostics, &mut corrections);
    }

    fn visit(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: &mut Option<&mut Vec<Correction>>,
    ) {
        let kind = node.kind();
        for &i in &self.call_all {
            self.cops[i].check_node(
                source,
                node,
                self.configs[i],
                diagnostics,
                corrections.as_deref_mut(),
            );
        }
        if let Some(idxs) = self.kinds.get(kind) {
            for &i in idxs {
                self.cops[i].check_node(
                    source,
                    node,
                    self.configs[i],
                    diagnostics,
                    corrections.as_deref_mut(),
                );
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit(source, child, diagnostics, corrections);
        }
    }
}
