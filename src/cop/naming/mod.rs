mod ascii_identifiers;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(ascii_identifiers::AsciiIdentifiers));
}
