//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_array_intersect(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_block_delimiters(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_class_methods_definitions(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_comparable_clamp(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_concat_array_literals(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_conditional_assignment(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_each_for_simple_loop(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_each_with_object(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_endless_method(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_exact_regexp_match(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_explicit_block_argument(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_format_string(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_identical_conditional_branches(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_if_inside_else(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_if_with_boolean_literal_branches(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_invertible_unless_condition(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_line_end_concatenation(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_magic_comment_format(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_method_call_with_args_parentheses(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_min_max_comparison(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_missing_respond_to_missing(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_multiline_memoization(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_nested_parenthesized_calls(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_optional_arguments(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_parallel_assignment(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_redundant_line_continuation(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_redundant_parentheses(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_redundant_regexp_argument(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_redundant_string_escape(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_return_nil_in_predicate_method_definition(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_semicolon(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_single_line_do_end_block(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_single_line_methods(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_symbol_proc(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_trailing_body_on_class(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_trailing_body_on_module(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_trivial_accessors(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

pub fn matches_while_until_modifier(_source: &SourceFile, _node: Node<'_>, _config: &CopConfig) -> bool { false }

