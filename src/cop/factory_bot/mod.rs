mod attribute_defined_statically;
mod create_list;
mod factory_class_name;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(attribute_defined_statically::AttributeDefinedStatically));
    registry.register(Box::new(create_list::CreateList));
    registry.register(Box::new(factory_class_name::FactoryClassName));
}
