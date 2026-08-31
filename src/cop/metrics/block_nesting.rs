//! Metrics/BlockNesting — nested control structures exceed Max.

use tree_sitter::{Node, Tree};

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct BlockNesting;

const NESTING: &[&str] = &[
    "if",
    "unless",
    "case",
    "while",
    "until",
    "for",
    "begin",
    "rescue",
];

impl Cop for BlockNesting {
    fn name(&self) -> &'static str {
        "Metrics/BlockNesting"
    }

    fn check_source(
        &self,
        source: &SourceFile,
        tree: &Tree,
        _code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let max = config.get_usize("Max", 3);
        let count_blocks = config.get_bool("CountBlocks", false);
        let count_modifiers = config.get_bool("CountModifierForms", false);
        visit(
            source,
            tree.root_node(),
            0,
            max,
            count_blocks,
            count_modifiers,
            self,
            diagnostics,
        );
    }
}

fn visit(
    source: &SourceFile,
    node: Node<'_>,
    depth: usize,
    max: usize,
    count_blocks: bool,
    count_modifiers: bool,
    cop: &BlockNesting,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let next = if is_nest(node.kind(), count_blocks, count_modifiers) {
        let new_depth = depth + 1;
        if new_depth > max {
            let (line, column) = source.offset_to_line_col(node.start_byte());
            diagnostics.push(cop.diagnostic(
                source,
                line,
                column,
                format!("Avoid more than {max} levels of block nesting. [{new_depth}/{max}]"),
            ));
        }
        new_depth
    } else {
        depth
    };
    let mut cur = node.walk();
    for child in node.children(&mut cur) {
        visit(source, child, next, max, count_blocks, count_modifiers, cop, diagnostics);
    }
}

fn is_nest(kind: &str, count_blocks: bool, count_modifiers: bool) -> bool {
    NESTING.contains(&kind)
        || (count_blocks && matches!(kind, "block" | "do_block"))
        || (count_modifiers
            && matches!(
                kind,
                "if_modifier" | "unless_modifier" | "while_modifier" | "until_modifier"
            ))
}
