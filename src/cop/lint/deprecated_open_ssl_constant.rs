use tree_sitter::Node;

use crate::cop::shared::{call_method_name, call_receiver, node_bytes, node_text};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/DeprecatedOpenSSLConstant — OpenSSL::Cipher::X.new etc.
pub struct DeprecatedOpenSSLConstant;

fn scope_parts(source: &SourceFile, mut node: Node<'_>) -> Vec<Vec<u8>> {
    let mut parts = Vec::new();
    loop {
        match node.kind() {
            "scope_resolution" => {
                if let Some(name) = node.child_by_field_name("name") {
                    parts.push(node_bytes(source, name).to_vec());
                }
                match node.child_by_field_name("scope") {
                    Some(s) => node = s,
                    None => break,
                }
            }
            "constant" => {
                parts.push(node_bytes(source, node).to_vec());
                break;
            }
            _ => break,
        }
    }
    parts.reverse();
    parts
}

fn replacement(mid: &[u8], meth: &[u8], algo: &str) -> String {
    if mid == b"Cipher" {
        format!("OpenSSL::Cipher.new(\"{algo}\")")
    } else if meth == b"digest" {
        format!("OpenSSL::Digest.digest(\"{algo}\", ...)")
    } else {
        format!("OpenSSL::Digest.new(\"{algo}\")")
    }
}

fn openssl_algo(parts: &[Vec<u8>]) -> Option<(&str, &[u8])> {
    if parts.len() < 3 || parts[0] != b"OpenSSL" {
        return None;
    }
    let mid = parts[1].as_slice();
    if mid != b"Cipher" && mid != b"Digest" {
        return None;
    }
    if mid == b"Digest" && parts.len() == 3 && parts[2] == b"Digest" {
        return None;
    }
    Some((std::str::from_utf8(&parts[2]).unwrap_or(""), mid))
}

impl Cop for DeprecatedOpenSSLConstant {
    fn name(&self) -> &'static str {
        "Lint/DeprecatedOpenSSLConstant"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["call"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let Some(meth) = call_method_name(source, node) else {
            return;
        };
        if meth != b"new" && meth != b"digest" {
            return;
        }
        let Some(recv) = call_receiver(node) else {
            return;
        };
        let parts = scope_parts(source, recv);
        let Some((algo, mid)) = openssl_algo(&parts) else {
            return;
        };
        let original = node_text(source, node);
        let repl = replacement(mid, meth, algo);
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            format!("Use `{repl}` instead of `{original}`."),
        ));
    }
}
