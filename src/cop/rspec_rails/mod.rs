mod http_status;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(http_status::HttpStatus));
}
