//! Run a single cop over fixture source (all lint phases).

use crate::cop::walker::BatchedWalker;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Run line + source + node phases for one cop (no message annotation / clang enrich).
pub fn run_cop_full(cop: &dyn Cop, source_bytes: &[u8]) -> Vec<Diagnostic> {
    run_cop_full_with_config(cop, source_bytes, CopConfig::default())
}

pub fn run_cop_full_with_config(
    cop: &dyn Cop,
    source_bytes: &[u8],
    config: CopConfig,
) -> Vec<Diagnostic> {
    run_cop_full_internal(cop, source_bytes, config, "test.rb")
}

pub fn run_cop_full_internal(
    cop: &dyn Cop,
    source_bytes: &[u8],
    config: CopConfig,
    filename: &str,
) -> Vec<Diagnostic> {
    let source = SourceFile::from_bytes(filename, source_bytes.to_vec());
    let tree = parse::parse_ruby(&source).expect("parse fixture");
    let code_map = CodeMap::from_tree(tree.root_node(), source.as_bytes());
    let mut diagnostics = Vec::new();
    cop.check_lines(&source, &config, &mut diagnostics, None);
    cop.check_source(&source, &tree, &code_map, &config, &mut diagnostics, None);
    BatchedWalker::new(vec![cop], vec![&config]).walk(
        &source,
        tree.root_node(),
        &mut diagnostics,
        None,
    );
    diagnostics
}
