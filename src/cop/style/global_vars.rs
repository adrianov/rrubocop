//! Style/GlobalVars — avoid non-built-in global variables.

use tree_sitter::Node;

use crate::cop::shared::node_bytes;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct GlobalVars;

/// RuboCop `BUILT_IN_VARS` (+ English / JRuby aliases).
const BUILTIN_GLOBALS: &[&[u8]] = &[
    b"$!", b"$@", b"$;", b"$,", b"$/", b"$\\", b"$.", b"$_", b"$~", b"$=", b"$*", b"$$", b"$?",
    b"$:", b"$\"", b"$<", b"$>", b"$0", b"$&", b"$`", b"$'", b"$+", b"$1", b"$2", b"$3", b"$4",
    b"$5", b"$6", b"$7", b"$8", b"$9", b"$PROGRAM_NAME", b"$VERBOSE", b"$DEBUG", b"$LOAD_PATH",
    b"$LOADED_FEATURES", b"$stdin", b"$stdout", b"$stderr", b"$FILENAME", b"$SAFE", b"$-a",
    b"$-d", b"$-i", b"$-l", b"$-p", b"$-v", b"$-w", b"$-0", b"$-F", b"$-I", b"$-K", b"$-W",
    b"$CHILD_STATUS", b"$ERROR_INFO", b"$ERROR_POSITION", b"$FIELD_SEPARATOR", b"$FS",
    b"$INPUT_LINE_NUMBER", b"$INPUT_RECORD_SEPARATOR", b"$LAST_MATCH_INFO", b"$LAST_PAREN_MATCH",
    b"$LAST_READ_LINE", b"$MATCH", b"$NR", b"$OFS", b"$ORS", b"$OUTPUT_FIELD_SEPARATOR",
    b"$OUTPUT_RECORD_SEPARATOR", b"$PID", b"$POSTMATCH", b"$PREMATCH", b"$PROCESS_ID", b"$RS",
    b"$DEFAULT_OUTPUT", b"$DEFAULT_INPUT", b"$IGNORECASE", b"$ARGV", b"$CLASSPATH",
    b"$JRUBY_VERSION", b"$JRUBY_REVISION", b"$ENV_JAVA",
];

impl Cop for GlobalVars {
    fn name(&self) -> &'static str {
        "Style/GlobalVars"
    }

    fn interested_node_kinds(&self) -> &'static [&'static str] {
        &["global_variable"]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: Node<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let name = node_bytes(source, node);
        if is_allowed(name, config) {
            return;
        }
        let (line, col) = source.offset_to_line_col(node.start_byte());
        diagnostics.push(self.diagnostic(
            source,
            line,
            col,
            "Do not introduce global variables.".to_string(),
        ));
    }
}

fn is_allowed(name: &[u8], config: &CopConfig) -> bool {
    BUILTIN_GLOBALS.contains(&name) || allowed_variables(config, name)
}

fn allowed_variables(config: &CopConfig, name: &[u8]) -> bool {
    let Some(allowed) = config.options.get("AllowedVariables") else {
        return false;
    };
    match allowed {
        serde_yml::Value::Sequence(items) => items.iter().any(|v| {
            v.as_str().is_some_and(|s| {
                let b = s.as_bytes();
                b == name || (name.starts_with(b"$") && b == &name[1..])
            })
        }),
        serde_yml::Value::String(s) => {
            let b = s.as_bytes();
            b == name || (name.starts_with(b"$") && b == &name[1..])
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(GlobalVars, "cops/style/global_vars");
}
