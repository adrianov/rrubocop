//! GraphQL/ExtractInputType — too many arguments; consider an input type.

use tree_sitter::Node;

use super::helpers::{class_body_stmts, is_argument_call, nested_class};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ExtractInputType;

impl Cop for ExtractInputType {
    fn name(&self) -> &'static str {
        "GraphQL/ExtractInputType"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/graphql/mutations/**/*.rb"]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if nested_class(node) {
            return;
        }
        let max = config.get_usize("MaxArguments", 2);
        for excess in class_body_stmts(node)
            .into_iter()
            .filter(|n| is_argument_call(source, *n))
            .skip(max)
        {
            let (line, col) = source.offset_to_line_col(excess.start_byte());
            diagnostics.push(self.diagnostic(
                source,
                line,
                col,
                "Consider moving arguments to a new input type".into(),
            ));
        }
    }
}
