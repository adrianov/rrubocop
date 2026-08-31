//! Heuristic matchers for breadth-first Style cops.
use tree_sitter::Node;
use crate::cop::CopConfig;
use crate::parse::source::SourceFile;

pub fn matches_if_with_semicolon(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "if" && node.kind() != "unless" { return false; }
    // `if cond; body` — then child starts with ;
    let mut cur = node.walk();
    let Some(then_n) = node.children(&mut cur).find(|ch| ch.kind() == "then") else { return false; };
    let b = &source.as_bytes()[then_n.start_byte()..then_n.end_byte()];
    b.starts_with(b";")
}

pub fn matches_in_pattern_then(_source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "in" { return false; }
    if node.start_position().row == node.end_position().row { return false; }
    let mut cur = node.walk();
    node.children(&mut cur).any(|ch| ch.kind() == "then")
}

pub fn matches_lambda_call(_source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    let style = config.get_str("EnforcedStyle", "call");
    match style {
        "call" => {
            // f.(args) — call with empty method name
            if node.kind() != "call" { return false; }
            node.child_by_field_name("method").is_none() && node.child_by_field_name("arguments").is_some()
        }
        "brackets" => node.kind() == "element_reference",
        "semantic" => false,
        _ => false,
    }
}

pub fn matches_multiline_if_then(_source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if !matches!(node.kind(), "if" | "unless") { return false; }
    if node.start_position().row == node.end_position().row { return false; }
    let mut cur = node.walk();
    let Some(then_n) = node.children(&mut cur).find(|ch| ch.kind() == "then") else { return false; };
    let mut c2 = then_n.walk();
    then_n.children(&mut c2).any(|ch| !ch.is_named() && ch.kind() == "then")
}

pub fn matches_non_nil_check(source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    // !x.nil?
    if node.kind() != "unary" { return false; }
    let mut cur = node.walk();
    if !node.children(&mut cur).any(|ch| !ch.is_named() && ch.kind() == "!") { return false; }
    let mut c2 = node.walk();
    let Some(operand) = node.named_children(&mut c2).next() else { return false; };
    operand.kind() == "call" && crate::cop::shared::call_method_name(source, operand) == Some(b"nil?")
}

pub fn matches_one_line_conditional(_source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if !matches!(node.kind(), "if" | "unless") { return false; }
    if node.start_position().row != node.end_position().row { return false; }
    let mut cur = node.walk();
    node.children(&mut cur).any(|ch| ch.kind() == "else")
}

pub fn matches_stabby_lambda_parentheses(source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "lambda" { return false; }
    let style = config.get_str("EnforcedStyle", "require_parentheses");
    let Some(params) = node.child_by_field_name("parameters") else {
        return style == "require_parentheses"; // ->{} without params may need ()
    };
    let bytes = &source.as_bytes()[params.start_byte()..params.end_byte()];
    let has_parens = bytes.starts_with(b"(");
    match style {
        "require_parentheses" => !has_parens,
        "require_no_parentheses" => has_parens,
        _ => false,
    }
}

pub fn matches_ternary_parentheses(_source: &SourceFile, node: Node<'_>, config: &CopConfig) -> bool {
    if node.kind() != "conditional" { return false; }
    let style = config.get_str("EnforcedStyle", "no_parentheses");
    let mut cur = node.walk();
    let first = node.named_children(&mut cur).next();
    let paren = first.is_some_and(|n| n.kind() == "parenthesized_statements");
    match style {
        "no_parentheses" => paren,
        "require_parentheses" => !paren,
        _ => false,
    }
}

pub fn matches_when_then(_source: &SourceFile, node: Node<'_>, _config: &CopConfig) -> bool {
    if node.kind() != "when" { return false; }
    if node.start_position().row == node.end_position().row { return false; }
    let mut cur = node.walk();
    node.children(&mut cur).any(|ch| ch.kind() == "then")
}

