//! FactoryBot/ExcessiveCreateList — `create_list(:name, N)` with N above MaxAmount.

use tree_sitter::Node;

use crate::cop::shared::{argument_nodes, call_receiver, node_bytes};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ExcessiveCreateList;

/// RuboCop `factory_call?`: no receiver (unless `ExplicitOnly`) or `FactoryBot` / `FactoryGirl`.
fn factory_call(source: &SourceFile, recv: Option<Node<'_>>, explicit_only: bool) -> bool {
    let Some(recv) = recv else {
        return !explicit_only;
    };
    let name = match recv.kind() {
        "constant" => recv,
        "scope_resolution" if recv.child_by_field_name("scope").is_none() => {
            match recv.child_by_field_name("name") {
                Some(n) => n,
                None => return false,
            }
        }
        _ => return false,
    };
    matches!(node_bytes(source, name), b"FactoryBot" | b"FactoryGirl")
}

/// Plain `:sym` / `"str"` (no interpolation), as RuboCop's `{sym str}`.
fn plain_name(node: Node<'_>) -> bool {
    let mut cur = node.walk();
    matches!(node.kind(), "simple_symbol" | "delimited_symbol" | "string")
        && !node
            .named_children(&mut cur)
            .any(|c| c.kind() == "interpolation")
}

/// Ruby integer literal value; `None` when it overflows (always excessive).
fn int_value(text: &[u8]) -> Option<u128> {
    let digits: String = String::from_utf8_lossy(text)
        .replace('_', "")
        .to_ascii_lowercase();
    let (radix, body) = match digits.as_bytes() {
        [b'0', b'x', ..] => (16, &digits[2..]),
        [b'0', b'b', ..] => (2, &digits[2..]),
        [b'0', b'o', ..] => (8, &digits[2..]),
        [b'0', b'd', ..] => (10, &digits[2..]),
        [b'0', _, ..] => (8, &digits[1..]),
        _ => (10, &digits[..]),
    };
    u128::from_str_radix(body, radix).ok()
}

/// Count literal of `create_list(:name, N, ...)` when N exceeds `max`.
fn excess_count<'a>(
    source: &SourceFile,
    node: Node<'a>,
    config: &CopConfig,
    max: usize,
) -> Option<Node<'a>> {
    if !factory_call(
        source,
        call_receiver(node),
        config.get_bool("ExplicitOnly", false),
    ) {
        return None;
    }
    let [name, count, ..] = argument_nodes(node)[..] else {
        return None;
    };
    let excessive = plain_name(name)
        && count.kind() == "integer"
        && int_value(node_bytes(source, count)).is_none_or(|n| n > max as u128);
    excessive.then_some(count)
}

impl Cop for ExcessiveCreateList {
    fn name(&self) -> &'static str {
        "FactoryBot/ExcessiveCreateList"
    }

    fn default_include(&self) -> &'static [&'static str] {
        &[
            "**/*_spec.rb",
            "**/spec/**/*",
            "**/test/**/*",
            "**/features/support/factories/**/*.rb",
        ]
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "command_call"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"create_list"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let max = config.get_usize("MaxAmount", 10);
        let Some(count) = excess_count(source, node, config, max) else {
            return;
        };
        let (line, col) = source.offset_to_line_col(count.start_byte());
        let mut diag = self.diagnostic(
            source,
            line,
            col,
            format!("Avoid using `create_list` with more than {max} items."),
        );
        diag.highlight_length = count.end_byte() - count.start_byte();
        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        ExcessiveCreateList,
        "cops/factory_bot/excessive_create_list"
    );
}
