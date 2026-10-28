//! Performance/BigDecimalWithNumericArgument — `BigDecimal(1.2)` / `1.2.to_d` want a
//! string; `BigDecimal('1')` / `'1'.to_d` want an integer (Ruby 3.1+).

use tree_sitter::Node;

use crate::cop::shared::{
    argument_nodes, call_method_name, call_receiver, named_kids, node_bytes, node_text,
    push_replace,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct BigDecimalWithNumericArgument;

/// Offending literal plus the byte range RuboCop rewrites.
struct Hit<'a> {
    lit: Node<'a>,
    start: usize,
    end: usize,
    replacement: String,
}

fn is_float(lit: Node<'_>) -> bool {
    match lit.kind() {
        "float" => true,
        "unary" => lit
            .child_by_field_name("operand")
            .is_some_and(|op| op.kind() == "float"),
        _ => false,
    }
}

/// Plain string literal whose whole value is ASCII digits.
fn digit_string<'a>(source: &'a SourceFile, lit: Node<'_>) -> Option<&'a str> {
    if lit.kind() != "string" {
        return None;
    }
    let [content] = named_kids(lit)[..] else {
        return None;
    };
    let text = node_bytes(source, content);
    (content.kind() == "string_content" && text.iter().all(u8::is_ascii_digit))
        .then(|| std::str::from_utf8(text).ok())
        .flatten()
}

fn quoted(source: &SourceFile, lit: Node<'_>) -> String {
    format!("'{}'", node_text(source, lit))
}

/// `BigDecimal(lit, ...)`: rewrite the literal in place.
fn big_decimal_hit<'a>(source: &SourceFile, node: Node<'a>) -> Option<Hit<'a>> {
    if call_receiver(node).is_some() || call_method_name(source, node)? != b"BigDecimal" {
        return None;
    }
    let lit = *argument_nodes(node).first()?;
    let replacement = if is_float(lit) {
        quoted(source, lit)
    } else {
        digit_string(source, lit)?.to_string()
    };
    Some(Hit {
        lit,
        start: lit.start_byte(),
        end: lit.end_byte(),
        replacement,
    })
}

/// `1.5.to_d(args)` → `BigDecimal('1.5', args)`.
fn big_decimal_call(source: &SourceFile, lit: Node<'_>, node: Node<'_>) -> String {
    let args: Vec<String> = std::iter::once(quoted(source, lit))
        .chain(argument_nodes(node).iter().map(|a| node_text(source, *a)))
        .collect();
    format!("BigDecimal({})", args.join(", "))
}

/// `lit.to_d(...)`: rewrite the whole call.
fn to_d_hit<'a>(source: &SourceFile, node: Node<'a>) -> Option<Hit<'a>> {
    let lit = call_receiver(node)?;
    if node
        .child_by_field_name("operator")
        .is_some_and(|op| node_bytes(source, op) == b"&.")
        || call_method_name(source, node)? != b"to_d"
    {
        return None;
    }
    let replacement = if is_float(lit) {
        big_decimal_call(source, lit, node)
    } else {
        format!("{}.to_d", digit_string(source, lit)?)
    };
    Some(Hit {
        lit,
        start: node.start_byte(),
        end: node.end_byte(),
        replacement,
    })
}

impl Cop for BigDecimalWithNumericArgument {
    fn name(&self) -> &'static str {
        "Performance/BigDecimalWithNumericArgument"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call", "command", "command_call"]
    }

    fn interested_call_names(&self) -> &'static [&'static [u8]] {
        &[b"BigDecimal", b"to_d"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if config.get_f64("TargetRubyVersion", 3.4) < 3.1 {
            return;
        }
        let Some(hit) = big_decimal_hit(source, node).or_else(|| to_d_hit(source, node)) else {
            return;
        };
        let msg = if is_float(hit.lit) {
            "Convert float literal to string and pass it to `BigDecimal`."
        } else {
            "Convert string literal to integer and pass it to `BigDecimal`."
        };
        let (line, col) = source.offset_to_line_col(hit.lit.start_byte());
        let mut diag = self.diagnostic(source, line, col, msg.into());
        diag.highlight_length = hit.lit.end_byte() - hit.lit.start_byte();
        diag.corrected = push_replace(
            &mut corrections,
            hit.start,
            hit.end,
            hit.replacement,
            self.name(),
        );
        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    crate::cop_fixture_tests!(
        BigDecimalWithNumericArgument,
        "cops/performance/big_decimal_with_numeric_argument"
    );

    fn corrected(src: &str) -> Vec<u8> {
        let source = SourceFile::from_bytes("t.rb", src.into());
        let tree = crate::parse::parse_ruby(&source).expect("parse");
        let mut corrections = Vec::new();
        crate::cop::walker::BatchedWalker::new(
            vec![&BigDecimalWithNumericArgument],
            vec![&CopConfig::default()],
        )
        .walk(
            &source,
            tree.root_node(),
            &mut Vec::new(),
            Some(&mut corrections),
        );
        crate::correction::CorrectionSet::from_vec(corrections).apply(source.as_bytes())
    }

    #[test]
    fn autocorrect_matches_rubocop() {
        for (bad, good) in [
            ("BigDecimal('0')\n", "BigDecimal(0)\n"),
            ("BigDecimal(\"4\", 6)\n", "BigDecimal(4, 6)\n"),
            (
                "BigDecimal(1.2, 3, exception: true)\n",
                "BigDecimal('1.2', 3, exception: true)\n",
            ),
            ("BigDecimal(-1.5)\n", "BigDecimal('-1.5')\n"),
            (
                "4.5.to_d(6, exception: true)\n",
                "BigDecimal('4.5', 6, exception: true)\n",
            ),
            ("'4'.to_d(6)\n", "4.to_d\n"),
        ] {
            assert_eq!(corrected(bad), good.as_bytes(), "{bad}");
        }
    }

    #[test]
    fn skipped_before_ruby_31() {
        let config = CopConfig {
            options: HashMap::from([(
                "TargetRubyVersion".into(),
                serde_yml::Value::Number(serde_yml::Number::from(3.0)),
            )]),
            ..CopConfig::default()
        };
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &BigDecimalWithNumericArgument,
            b"BigDecimal('1')\n",
            config,
        );
    }
}
