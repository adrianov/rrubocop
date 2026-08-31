//! Rails/ApplicationMailer — breadth-first tree-sitter port.

use tree_sitter::Node;

use super::enforce_superclass::check_superclass;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ApplicationMailer;

impl Cop for ApplicationMailer {
    fn name(&self) -> &'static str {
        "Rails/ApplicationMailer"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["app/mailers/**/*.rb"]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn safe_autocorrect(&self) -> bool {
        false
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["class"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<Correction>>,
    ) {
        check_superclass(
            self,
            source,
            node,
            "ActionMailer::Base",
            "ApplicationMailer",
            "Use `ApplicationMailer` instead of `ActionMailer::Base`.",
            diagnostics,
            corrections,
        );
    }
}
