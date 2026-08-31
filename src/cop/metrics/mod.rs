mod abc_size;
mod block_nesting;
mod collection_literal_length;
mod parameter_lists;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(abc_size::AbcSize));
    registry.register(Box::new(block_nesting::BlockNesting));
    registry.register(Box::new(collection_literal_length::CollectionLiteralLength));
    registry.register(Box::new(parameter_lists::ParameterLists));
}
