//! Lint/ItWithoutArgumentsInBlock — no-op (Ruby 3.4+ `it` block param).

use crate::cop::Cop;
use crate::diagnostic::Severity;

pub struct ItWithoutArgumentsInBlock;

impl Cop for ItWithoutArgumentsInBlock {
    fn name(&self) -> &'static str {
        "Lint/ItWithoutArgumentsInBlock"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
}
