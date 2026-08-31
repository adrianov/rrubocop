//! Per-cop YAML → CopConfig.

use serde_yml::Value;

use crate::cop::{CopConfig, EnabledState};
use crate::diagnostic::Severity;

use super::parse::value_to_string_list;

pub(crate) fn parse_cop_config(value: &Value) -> CopConfig {
    let mut config = CopConfig::default();
    if let Value::Mapping(map) = value {
        for (k, v) in map {
            if let Some(key) = k.as_str() {
                apply_cop_key(&mut config, key, v);
            }
        }
    }
    config
}

fn apply_cop_key(config: &mut CopConfig, key: &str, v: &Value) {
    match key {
        "Enabled" => {
            if let Some(state) = parse_enabled_state(v) {
                config.enabled = state;
            }
        }
        "Severity" => {
            if let Some(s) = v.as_str() {
                config.severity = Severity::from_str(s);
            }
        }
        "Exclude" => {
            if let Some(list) = value_to_string_list(v) {
                config.exclude = list;
            }
        }
        "Include" => {
            if let Some(list) = value_to_string_list(v) {
                config.include = list;
            }
        }
        _ => {
            config.options.insert(key.to_string(), v.clone());
        }
    }
}

pub(crate) fn parse_enabled_state(v: &Value) -> Option<EnabledState> {
    if let Some(b) = v.as_bool() {
        Some(if b {
            EnabledState::True
        } else {
            EnabledState::False
        })
    } else if v.as_str() == Some("pending") {
        Some(EnabledState::Pending)
    } else {
        None
    }
}
