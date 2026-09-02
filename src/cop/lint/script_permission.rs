//! Lint/ScriptPermission — shebang scripts must be executable (non-Windows).

use tree_sitter::Tree;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub struct ScriptPermission;

impl Cop for ScriptPermission {
    fn name(&self) -> &'static str {
        "Lint/ScriptPermission"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn uses_source_phase(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        _tree: &Tree,
        _code_map: &CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if cfg!(windows) {
            return;
        }
        if !source.as_bytes().starts_with(b"#!") {
            return;
        }
        let Ok(meta) = std::fs::metadata(&source.path) else {
            return;
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if meta.permissions().mode() & 0o111 != 0 {
                return;
            }
        }
        #[cfg(not(unix))]
        {
            return;
        }
        let basename = source
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        diagnostics.push(self.diagnostic(
            source,
            1,
            0,
            format!("Script file {basename} doesn't have execute permission."),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use crate::testutil::parse_fixture;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    fn run_on_path(path: &Path) -> Vec<Diagnostic> {
        let source = SourceFile::from_path(path).expect("read script");
        let tree = parse::parse_ruby(&source).expect("parse");
        let code_map = CodeMap::from_tree(tree.root_node(), source.as_bytes());
        let mut diagnostics = Vec::new();
        ScriptPermission.check_source(
            &source,
            &tree,
            &code_map,
            &CopConfig::default(),
            &mut diagnostics,
            None,
        );
        diagnostics
    }

    fn write_script(dir: &Path, name: &str, content: &[u8], mode: u32) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
        path
    }

    fn run_fixture(raw: &[u8], mode: u32) -> Vec<Diagnostic> {
        let parsed = parse_fixture(raw);
        let name = parsed.filename.as_deref().unwrap_or("script.rb");
        let dir = tempfile::tempdir().unwrap();
        let path = write_script(dir.path(), name, &parsed.source, mode);
        run_on_path(&path)
    }

    #[test]
    #[cfg(unix)]
    fn offense_fixture() {
        let raw = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/cops/lint/script_permission/offense.rb"
        ));
        let parsed = parse_fixture(raw);
        let diags = run_fixture(raw, 0o644);
        assert_eq!(diags.len(), parsed.expected.len());
        for (diag, exp) in diags.iter().zip(parsed.expected.iter()) {
            assert_eq!(diag.location.line, exp.line);
            assert_eq!(diag.location.column, exp.column);
            assert_eq!(diag.cop_name, exp.cop_name);
            assert_eq!(diag.message, exp.message);
        }
    }

    #[test]
    #[cfg(unix)]
    fn no_offense_fixture() {
        let raw = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/cops/lint/script_permission/no_offense.rb"
        ));
        assert!(run_fixture(raw, 0o755).is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn offense_when_not_executable() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_script(
            dir.path(),
            "lifecycle_events.rb",
            b"#!/usr/bin/env ruby\nputs 'hello'\n",
            0o644,
        );
        let diags = run_on_path(&path);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].cop_name, "Lint/ScriptPermission");
        assert_eq!(diags[0].location.line, 1);
        assert_eq!(
            diags[0].message,
            "Script file lifecycle_events.rb doesn't have execute permission."
        );
    }

    #[test]
    #[cfg(unix)]
    fn no_offense_when_executable() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_script(
            dir.path(),
            "lifecycle_events.rb",
            b"#!/usr/bin/env ruby\nputs 'hello'\n",
            0o755,
        );
        assert!(run_on_path(&path).is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn no_offense_without_shebang() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_script(dir.path(), "plain.rb", b"puts 'hello'\n", 0o644);
        assert!(run_on_path(&path).is_empty());
    }
}
