//! AllCops YAML field parsing.

use serde_yml::{Mapping, Value};

use super::parse::extract_string_list;

/// Fields extracted from an `AllCops:` mapping.
#[derive(Default)]
pub(crate) struct AllCopsFields {
    pub(crate) global_excludes: Vec<String>,
    pub(crate) new_cops: Option<String>,
    pub(crate) disabled_by_default: Option<bool>,
    pub(crate) target_ruby_version: Option<f64>,
    pub(crate) target_rails_version: Option<f64>,
    pub(crate) active_support_extensions_enabled: Option<bool>,
    pub(crate) migrated_schema_version: Option<String>,
    pub(crate) display_cop_names: Option<bool>,
    pub(crate) display_style_guide: Option<bool>,
    pub(crate) extra_details: Option<bool>,
    pub(crate) style_guide_base_url: Option<String>,
}

/// Parse the `AllCops` top-level key into typed fields.
pub(crate) fn parse_allcops(value: &Value) -> AllCopsFields {
    let mut fields = AllCopsFields::default();
    if let Some(excludes) = extract_string_list(value, "Exclude") {
        fields.global_excludes = excludes;
    }
    if let Value::Mapping(ac_map) = value {
        apply_new_cops(&mut fields, ac_map);
        apply_disabled_by_default(&mut fields, ac_map);
        apply_target_versions(&mut fields, ac_map);
        apply_as_extensions(&mut fields, ac_map);
        apply_migrated_schema(&mut fields, ac_map);
        apply_display_options(&mut fields, ac_map);
    }
    fields
}

fn apply_display_options(fields: &mut AllCopsFields, ac_map: &Mapping) {
    if let Some(v) = map_get(ac_map, "DisplayCopNames") {
        fields.display_cop_names = v.as_bool();
    }
    if let Some(v) = map_get(ac_map, "DisplayStyleGuide") {
        fields.display_style_guide = v.as_bool();
    }
    if let Some(v) = map_get(ac_map, "ExtraDetails") {
        fields.extra_details = v.as_bool();
    }
    if let Some(v) = map_get(ac_map, "StyleGuideBaseURL") {
        fields.style_guide_base_url = v.as_str().map(String::from);
    }
}

fn map_get<'a>(ac_map: &'a Mapping, key: &str) -> Option<&'a Value> {
    ac_map.get(Value::String(key.to_string()))
}

fn apply_new_cops(fields: &mut AllCopsFields, ac_map: &Mapping) {
    if let Some(nc) = map_get(ac_map, "NewCops") {
        fields.new_cops = nc.as_str().map(String::from);
    }
}

fn apply_disabled_by_default(fields: &mut AllCopsFields, ac_map: &Mapping) {
    if let Some(dbd) = map_get(ac_map, "DisabledByDefault") {
        fields.disabled_by_default = dbd.as_bool();
    }
}

fn apply_target_versions(fields: &mut AllCopsFields, ac_map: &Mapping) {
    if let Some(trv) = map_get(ac_map, "TargetRubyVersion") {
        fields.target_ruby_version = parse_version_f64(trv);
    }
    if let Some(trv) = map_get(ac_map, "TargetRailsVersion") {
        fields.target_rails_version = parse_version_f64(trv);
    }
}

fn apply_as_extensions(fields: &mut AllCopsFields, ac_map: &Mapping) {
    if let Some(ase) = map_get(ac_map, "ActiveSupportExtensionsEnabled") {
        fields.active_support_extensions_enabled = ase.as_bool();
    }
}

fn apply_migrated_schema(fields: &mut AllCopsFields, ac_map: &Mapping) {
    if let Some(msv) = map_get(ac_map, "MigratedSchemaVersion") {
        fields.migrated_schema_version = parse_schema_version(msv);
    }
}

fn parse_version_f64(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_u64().map(|u| u as f64))
        .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
}

fn parse_schema_version(v: &Value) -> Option<String> {
    v.as_str()
        .map(String::from)
        .or_else(|| v.as_u64().map(|u| u.to_string()))
        .or_else(|| v.as_i64().map(|i| i.to_string()))
}
