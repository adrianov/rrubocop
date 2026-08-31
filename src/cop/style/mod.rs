mod and_or;
mod empty_literal;
mod frozen_string_literal_comment;
mod not;
mod redundant_begin;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(
        frozen_string_literal_comment::FrozenStringLiteralComment,
    ));
    registry.register(Box::new(redundant_begin::RedundantBegin));
    registry.register(Box::new(empty_literal::EmptyLiteral));
    registry.register(Box::new(and_or::AndOr));
    registry.register(Box::new(not::Not));
}
