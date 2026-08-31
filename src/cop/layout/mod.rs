mod empty_lines;
mod end_of_line;
mod leading_empty_lines;
mod space_after_comma;
mod space_before_comma;
mod trailing_empty_lines;
mod trailing_whitespace;

use crate::cop::registry::CopRegistry;

pub fn register_all(registry: &mut CopRegistry) {
    registry.register(Box::new(trailing_whitespace::TrailingWhitespace));
    registry.register(Box::new(trailing_empty_lines::TrailingEmptyLines));
    registry.register(Box::new(end_of_line::EndOfLine));
    registry.register(Box::new(leading_empty_lines::LeadingEmptyLines));
    registry.register(Box::new(empty_lines::EmptyLines));
    registry.register(Box::new(space_after_comma::SpaceAfterComma));
    registry.register(Box::new(space_before_comma::SpaceBeforeComma));
}
