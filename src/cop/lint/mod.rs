mod useless_assignment;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(useless_assignment::UselessAssignment));
}
