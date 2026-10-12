//! Rails/RedundantActiveRecordAllMethod — `all` before an Active Record query.

use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, node_bytes, push_replace};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::model::FileModel;
use crate::parse::source::SourceFile;

pub struct RedundantActiveRecordAllMethod;

const MSG: &str = "Redundant `all` detected.";

mod decide;

use decide::{All, Hit, QUERYING, call_all, is_querying, keep};

fn report(
    cop: &RedundantActiveRecordAllMethod,
    source: &SourceFile,
    hit: Hit<'_>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let (line, col) = source.offset_to_line_col(hit.start);
    let mut diag = cop.diagnostic(source, line, col, MSG.to_string());
    diag.highlight_length = hit.end.saturating_sub(hit.start).max(1);
    if push_replace(corrections, hit.start, hit.end, "", cop.name())
        && push_replace(
            corrections,
            hit.op.start_byte(),
            hit.op.end_byte(),
            "",
            cop.name(),
        )
    {
        diag.corrected = true;
    }
    diagnostics.push(diag);
}

fn on_query(
    cop: &RedundantActiveRecordAllMethod,
    source: &SourceFile,
    query: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(recv) = call_receiver(query) else {
        return;
    };
    let Some(all) = call_all(source, recv) else {
        return;
    };
    let Some(hit) = keep(source, query, config, all) else {
        return;
    };
    report(cop, source, hit, diagnostics, corrections);
}

fn on_bare_all(
    cop: &RedundantActiveRecordAllMethod,
    source: &SourceFile,
    query: Node<'_>,
    recv: Node<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    if !call_method_name(source, query).is_some_and(is_querying) {
        return;
    }
    let all = All {
        start: recv.start_byte(),
        end: recv.end_byte(),
        receiver: None,
    };
    let Some(hit) = keep(source, query, config, all) else {
        return;
    };
    report(cop, source, hit, diagnostics, corrections);
}

fn scan_bare_all(
    cop: &RedundantActiveRecordAllMethod,
    source: &SourceFile,
    file_model: &FileModel<'_>,
    config: &CopConfig,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let root = file_model.tree.root_node();
    for &byte in &file_model.vcall_sites {
        let Some(recv) = root.descendant_for_byte_range(byte, byte) else {
            continue;
        };
        if recv.kind() != "identifier" || node_bytes(source, recv) != b"all" {
            continue;
        }
        let Some(query) = recv.parent().filter(|query| {
            call_receiver(*query).is_some_and(|r| r.start_byte() == recv.start_byte())
        }) else {
            continue;
        };
        on_bare_all(cop, source, query, recv, config, diagnostics, corrections);
    }
}

impl Cop for RedundantActiveRecordAllMethod {
    fn name(&self) -> &'static str {
        "Rails/RedundantActiveRecordAllMethod"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn safe_autocorrect(&self) -> bool {
        false
    }

    fn needs_file_model(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "command_call"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        QUERYING
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        on_query(self, source, node, config, diagnostics, &mut corrections);
    }

    fn check_file_model(
        &self,
        source: &SourceFile,
        file_model: &FileModel<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        scan_bare_all(
            self,
            source,
            file_model,
            config,
            diagnostics,
            &mut corrections,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correction::CorrectionSet;

    crate::cop_fixture_tests!(
        RedundantActiveRecordAllMethod,
        "cops/rails/redundant_active_record_all_method"
    );

    fn collect(source: &SourceFile, tree: &tree_sitter::Tree) -> Vec<Correction> {
        let fm = crate::model::build(source.as_bytes(), tree.clone());
        let config = CopConfig::default();
        let mut diagnostics = Vec::new();
        let mut corrections = Vec::new();
        RedundantActiveRecordAllMethod.check_file_model(
            source,
            &fm,
            &config,
            &mut diagnostics,
            Some(&mut corrections),
        );
        crate::cop::walker::BatchedWalker::new(
            vec![&RedundantActiveRecordAllMethod],
            vec![&config],
        )
        .walk(
            source,
            tree.root_node(),
            &mut diagnostics,
            Some(&mut corrections),
        );
        corrections
    }

    fn rewritten(src: &str) -> String {
        let source = SourceFile::from_bytes("test.rb", src.as_bytes().to_vec());
        String::from_utf8(
            CorrectionSet::from_vec(collect(
                &source,
                &crate::parse::parse_ruby(&source).expect("parse"),
            ))
            .apply(source.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn querying_methods_are_sorted() {
        assert!(QUERYING.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn autocorrect_drops_all_and_dot() {
        assert_eq!(
            rewritten("User.all.order(:created_at)\n"),
            "User.order(:created_at)\n"
        );
        assert_eq!(
            rewritten("User.all().order(:created_at)\n"),
            "User.order(:created_at)\n"
        );
        assert_eq!(
            rewritten("user.articles.all.order(:created_at)\n"),
            "user.articles.order(:created_at)\n"
        );
    }

    #[test]
    fn autocorrect_bare_all_inside_model() {
        let src =
            "class User < ApplicationRecord\n  scope :admins, -> { all.where(admin: true) }\nend\n";
        let want =
            "class User < ApplicationRecord\n  scope :admins, -> { where(admin: true) }\nend\n";
        assert_eq!(rewritten(src), want);
    }

    #[test]
    fn autocorrect_find_each_block() {
        let src = "WidgetPosition.all.find_each do |widget_position|\nend\n";
        let want = "WidgetPosition.find_each do |widget_position|\nend\n";
        assert_eq!(rewritten(src), want);
    }

    #[test]
    fn autocorrect_is_unsafe() {
        assert!(!RedundantActiveRecordAllMethod.safe_autocorrect());
    }
}
