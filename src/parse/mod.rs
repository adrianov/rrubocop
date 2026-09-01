pub mod codemap;
pub mod comment_hash;
pub mod directives;
pub mod source;

use anyhow::Result;
use tree_sitter::{Parser, Tree};

use self::source::SourceFile;

pub fn parse_ruby(source: &SourceFile) -> Result<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_ruby::LANGUAGE.into())
        .expect("ruby grammar");
    parser
        .parse(&source.content, None)
        .ok_or_else(|| anyhow::anyhow!("parse failed for {}", source.path.display()))
}
