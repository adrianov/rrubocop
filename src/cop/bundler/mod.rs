mod duplicated_gem;
mod duplicated_group;
mod gem_filename;
mod insecure_protocol_source;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(duplicated_gem::DuplicatedGem));
    registry.register(Box::new(duplicated_group::DuplicatedGroup));
    registry.register(Box::new(gem_filename::GemFilename));
    registry.register(Box::new(insecure_protocol_source::InsecureProtocolSource));
}
