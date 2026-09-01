//! Heuristic matchers for breadth-first Style cops.

mod calls;
pub use calls::matches_class_and_module_children;

mod collections;
pub use collections::matches_empty_heredoc;
pub use collections::matches_hash_syntax;
pub use collections::matches_symbol_array;
pub use collections::matches_word_array;

mod control;
pub use control::matches_if_with_semicolon;
pub use control::matches_one_line_conditional;
pub use control::matches_stabby_lambda_parentheses;
pub use control::matches_ternary_parentheses;

mod percent;
pub use percent::matches_bare_percent_literals;
pub use percent::matches_command_literal;
pub use percent::matches_percent_q_literals;

mod string_literals;
pub use string_literals::matches_string_literals;
pub(crate) use string_literals::double_quotes_required;

mod redundant_misc;
pub use redundant_misc::matches_redundant_capital_w;
pub use redundant_misc::matches_redundant_double_splat_hash_braces;
pub use redundant_misc::matches_redundant_heredoc_delimiter_quotes;
pub use redundant_misc::matches_redundant_interpolation;
pub use redundant_misc::matches_redundant_percent_q;

mod identical_branches;
pub use identical_branches::identical_branch_nodes;
#[allow(unused_imports)]
pub use identical_branches::matches_identical_conditional_branches;

mod stubs;
pub use stubs::matches_block_delimiters;
pub use stubs::matches_class_methods_definitions;
pub use stubs::matches_conditional_assignment;
pub use stubs::matches_explicit_block_argument;
pub use stubs::matches_if_inside_else;
pub use stubs::matches_if_with_boolean_literal_branches;
pub use stubs::matches_invertible_unless_condition;
pub use stubs::matches_line_end_concatenation;
pub use stubs::matches_method_call_with_args_parentheses;
pub use stubs::matches_multiline_memoization;
pub use stubs::matches_nested_parenthesized_calls;
pub use stubs::matches_optional_arguments;
pub use stubs::matches_redundant_line_continuation;
pub use stubs::matches_redundant_parentheses;
pub use stubs::matches_redundant_string_escape;
pub use stubs::matches_return_nil_in_predicate_method_definition;
pub use stubs::matches_symbol_proc;
pub use stubs::matches_while_until_modifier;
