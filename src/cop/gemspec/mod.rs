//! Gemspec department cops (*.gemspec).

mod deprecated_attribute_assignment;
mod duplicated_assignment;
mod ordered_dependencies;
mod ordered_dependencies_group;
mod ruby_version_globals_usage;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(deprecated_attribute_assignment::DeprecatedAttributeAssignment));
    registry.register(Box::new(duplicated_assignment::DuplicatedAssignment));
    registry.register(Box::new(ordered_dependencies::OrderedDependencies));
    registry.register(Box::new(ruby_version_globals_usage::RubyVersionGlobalsUsage));
}
