pub mod layout;
pub mod lint;
pub mod metrics;
pub mod naming;
pub mod registry;
pub mod security;
pub mod style;
pub mod walker;

use std::collections::HashMap;

use serde::Serialize;

use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

pub use registry::CopRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum EnabledState {
    True,
    False,
    Pending,
    #[default]
    Unset,
}

#[derive(Debug, Clone, Serialize)]
pub struct CopConfig {
    pub enabled: EnabledState,
    pub severity: Option<Severity>,
    pub exclude: Vec<String>,
    pub include: Vec<String>,
    pub options: HashMap<String, serde_yml::Value>,
}

impl Default for CopConfig {
    fn default() -> Self {
        Self {
            enabled: EnabledState::Unset,
            severity: None,
            exclude: Vec::new(),
            include: Vec::new(),
            options: HashMap::new(),
        }
    }
}

impl CopConfig {
    pub fn get_str<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.options
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.options
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    pub fn get_usize(&self, key: &str, default: usize) -> usize {
        self.options
            .get(key)
            .and_then(|v| {
                v.as_u64()
                    .map(|u| u as usize)
                    .or_else(|| v.as_i64().map(|i| i as usize))
                    .or_else(|| v.as_f64().map(|f| f as usize))
            })
            .unwrap_or(default)
    }

    pub fn get_f64(&self, key: &str, default: f64) -> f64 {
        self.options
            .get(key)
            .and_then(|v| v.as_f64().or_else(|| v.as_u64().map(|u| u as f64)))
            .unwrap_or(default)
    }
}

pub trait Cop: Send + Sync {
    fn name(&self) -> &'static str;

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn default_include(&self) -> &'static [&'static str] {
        &[]
    }

    fn default_exclude(&self) -> &'static [&'static str] {
        &[]
    }

    fn default_enabled(&self) -> bool {
        true
    }

    fn diagnostic(
        &self,
        source: &SourceFile,
        line: usize,
        column: usize,
        message: String,
    ) -> Diagnostic {
        Diagnostic {
            path: source.path_str().to_string(),
            location: Location { line, column },
            severity: self.default_severity(),
            cop_name: self.name().to_string(),
            message,
            corrected: false,
        }
    }

    fn supports_autocorrect(&self) -> bool {
        false
    }

    fn safe_autocorrect(&self) -> bool {
        true
    }

    #[allow(unused_variables)]
    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
    }

    #[allow(unused_variables)]
    fn check_source(
        &self,
        source: &SourceFile,
        tree: &tree_sitter::Tree,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
    }

    /// tree-sitter node kinds this cop cares about; empty = all nodes.
    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &[]
    }

    #[allow(unused_variables)]
    fn check_node(
        &self,
        source: &SourceFile,
        node: tree_sitter::Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
    }
}
