mod http_status;
mod inferred_spec_type;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(http_status::HttpStatus));
    registry.register(Box::new(inferred_spec_type::InferredSpecType));
}
