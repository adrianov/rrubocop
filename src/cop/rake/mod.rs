pub const RAKE_DEFAULT_INCLUDE: &[&str] = &["**/*.rake", "**/Rakefile", "Rakefile"];

mod class_definition_in_task;
mod desc;
mod duplicate_namespace;
mod duplicate_task;
mod method_definition_in_task;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(class_definition_in_task::ClassDefinitionInTask));
    registry.register(Box::new(desc::Desc));
    registry.register(Box::new(duplicate_namespace::DuplicateNamespace));
    registry.register(Box::new(duplicate_task::DuplicateTask));
    registry.register(Box::new(method_definition_in_task::MethodDefinitionInTask));
}
