//! Shared MCP lint session: cop registry plus optional test-only config.

use crate::config::{CopFilterSet, ResolvedConfig};
use crate::cop::registry::CopRegistry;

pub(crate) struct State {
    pub(crate) registry: CopRegistry,
    /// Built-in defaults only (MCP unit tests). Production resolves per `path`.
    pub(crate) fixed: Option<FixedLint>,
}

pub(crate) struct FixedLint {
    pub(crate) config: ResolvedConfig,
    pub(crate) filters: CopFilterSet,
}
