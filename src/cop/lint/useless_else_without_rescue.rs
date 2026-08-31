//! Lint/UselessElseWithoutRescue — no-op (syntax error in modern Ruby).

use crate::cop::Cop;
use crate::diagnostic::Severity;

pub struct UselessElseWithoutRescue;

impl Cop for UselessElseWithoutRescue {
    fn name(&self) -> &'static str {
        "Lint/UselessElseWithoutRescue"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
}
