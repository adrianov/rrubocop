//! Metrics/ParameterLists — too many method/block parameters.

use tree_sitter::Node;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ParameterLists;

impl Cop for ParameterLists {
    fn name(&self) -> &'static str {
        "Metrics/ParameterLists"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["method", "singleton_method", "block", "do_block", "lambda"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let max = config.get_usize("Max", 5);
        let count_kw = config.get_bool("CountKeywordArgs", true);
        let max_optional = config.get_usize("MaxOptionalParameters", 3);
        let Some(params) = node
            .child_by_field_name("parameters")
            .or_else(|| node.child_by_field_name("block_parameters"))
        else {
            return;
        };
        let (total, optional) = count_params(params, count_kw);
        report_if_over(self, source, params, total, max, "parameters", diagnostics);
        report_if_over(
            self,
            source,
            params,
            optional,
            max_optional,
            "optional arguments",
            diagnostics,
        );
    }
}

fn report_if_over(
    cop: &ParameterLists,
    source: &SourceFile,
    params: Node<'_>,
    count: usize,
    max: usize,
    kind: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if count <= max {
        return;
    }
    let (line, column) = source.offset_to_line_col(params.start_byte());
    let msg = if kind == "parameters" {
        format!("Avoid parameter lists longer than {max} parameters. [{count}/{max}]")
    } else {
        format!(
            "Avoid optional parameters that take longer than {max} arguments. [{count}/{max}]"
        )
    };
    diagnostics.push(cop.diagnostic(source, line, column, msg));
}

fn count_params(params: Node<'_>, count_kw: bool) -> (usize, usize) {
    let mut total = 0usize;
    let mut optional = 0usize;
    let mut cur = params.walk();
    for child in params.named_children(&mut cur) {
        match child.kind() {
            "identifier" | "destructured_parameter" | "splat_parameter" | "hash_splat_parameter" => {
                total += 1
            }
            "optional_parameter" => {
                total += 1;
                optional += 1;
            }
            "keyword_parameter" | "hash_splat_keyword_parameter" if count_kw => total += 1,
            _ => {}
        }
    }
    (total, optional)
}
