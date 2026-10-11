//! Style/ExplicitBlockArgument — pass the method's block instead of `yield`.

use tree_sitter::Node;

use crate::cop::style::heuristics::matches_explicit_block_argument;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ExplicitBlockArgument;

const MSG: &str =
    "Consider using explicit block argument in the surrounding method's signature over `yield`.";
pub(super) const COP: &str = "Style/ExplicitBlockArgument";

mod edit;
mod rewrite;

pub(super) fn host<'a>(node: Node<'a>) -> Node<'a> {
    match node.parent() {
        Some(p) if matches!(p.kind(), "call" | "command" | "command_call") => p,
        _ => node,
    }
}

impl Cop for ExplicitBlockArgument {
    fn name(&self) -> &'static str {
        COP
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["block", "do_block"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if !matches_explicit_block_argument(source, node, config) {
            return;
        }
        let at = host(node);
        let (line, col) = source.offset_to_line_col(at.start_byte());
        let mut diag = self.diagnostic(source, line, col, MSG.to_string());
        diag.highlight_length = at.end_byte().saturating_sub(at.start_byte()).max(1);
        if rewrite::rewrite(source, node, &mut corrections) {
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(ExplicitBlockArgument, "cops/style/explicit_block_argument");

    fn collect(
        src: &str,
    ) -> (
        crate::parse::source::SourceFile,
        Vec<crate::correction::Correction>,
    ) {
        let source = crate::parse::source::SourceFile::from_bytes("t.rb", src.as_bytes().to_vec());
        let tree = crate::parse::parse_ruby(&source).expect("parse");
        let config = CopConfig::default();
        let mut corrections = Vec::new();
        crate::cop::walker::BatchedWalker::new(vec![&ExplicitBlockArgument], vec![&config]).walk(
            &source,
            tree.root_node(),
            &mut Vec::new(),
            Some(&mut corrections),
        );
        (source, corrections)
    }

    fn corrected(src: &str) -> String {
        let (source, corrections) = collect(src);
        String::from_utf8(
            crate::correction::CorrectionSet::from_vec(corrections).apply(source.as_bytes()),
        )
        .expect("utf8")
    }

    #[test]
    fn inspection_is_correctable() {
        let diags = crate::testutil::run_cop_full(
            &ExplicitBlockArgument,
            b"def load_variants(group_type)\n  measure(operation:, group_type:) { yield }\nend\n",
        );
        assert_eq!(diags.len(), 1);
        assert!(diags[0].correctable);
        assert!(!diags[0].corrected);
    }

    #[test]
    fn autocorrect_passes_block() {
        let src = "\
def nine_times
  9.times { yield }
end

def with_tmp_dir
  Dir.mktmpdir do |tmp_dir|
    Dir.chdir(tmp_dir) { |dir| yield dir }
  end
end

def load_variants(group_type)
  measure(operation:, group_type:) { yield }
end

def pass_arg
  run { |x| yield x }
end

def block_do
  9.times do
    yield
  end
end

def already(&blk)
  bar { yield }
end

def empty_parens
  foo() { yield }
end

def two
  a { yield }
  b { yield }
end

def self.singleton
  tap { yield }
end

def outer
  class << self
    tap { yield }
  end
end

def command_args
  foo bar { yield }
end

def kwargs_nospace
  bar(a:1){ yield }
end

def multiline
  bar(
    a
  ) { yield }
end

def calltrail
  bar(a,) { yield }
end

def bare a
  bar { yield }
end

def anon(&)
  bar { yield }
end

def load_variants2(group_type)
  return unless applicable?

  Telemetry::Metrics::VirtualProducts.operation_duration.measure(operation:, group_type:) { yield }
rescue StandardError => e
  raise
end

def child(a, b = 1)
  super { yield }
end

def kw(j: 1, k:)
  super { yield }
end

def destruct((a, b))
  super { yield }
end

def endless = tap { yield }
";
        let want = "\
def nine_times(&block)
  9.times(&block)
end

def with_tmp_dir(&block)
  Dir.mktmpdir do |tmp_dir|
    Dir.chdir(tmp_dir, &block)
  end
end

def load_variants(group_type, &block)
  measure(operation:, group_type:, &block)
end

def pass_arg(&block)
  run(&block)
end

def block_do(&block)
  9.times(&block)
end

def already(&blk)
  bar(&blk)
end

def empty_parens(&block)
  foo(&block)
end

def two(&block)
  a(&block)
  b(&block)
end

def self.singleton(&block)
  tap(&block)
end

def outer(&block)
  class << self
    tap(&block)
  end
end

def command_args(&block)
  foo bar(&block)
end

def kwargs_nospace(&block)
  bar(a:1, &block)
end

def multiline(&block)
  bar(
    a, &block
  )
end

def calltrail(&block)
  bar(a, &block)
end

def bare a, &block
  bar(&block)
end

def anon(&)
  bar(&)
end

def load_variants2(group_type, &block)
  return unless applicable?

  Telemetry::Metrics::VirtualProducts.operation_duration.measure(operation:, group_type:, &block)
rescue StandardError => e
  raise
end

def child(a, b = 1, &block)
  super(a, b, &block)
end

def kw(j: 1, k:, &block)
  super(j: j, k:, &block)
end

def destruct((a, b), &block)
  super(a, b, &block)
end

def endless(&block) = tap(&block)
";
        assert_eq!(corrected(src), want);
    }
}
