//! RuboCop-compatible MCP server over stdio (`rrubocop --mcp`).
//!
//! Uses the official [`rmcp`] SDK. Tools match RuboCop 1.85+:
//! `rubocop_inspection` and `rubocop_autocorrection`.

mod io;
mod offense;
mod state;
mod targets;
mod tools;

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Result;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt};
use serde::Deserialize;
use tokio::runtime::Runtime;

use crate::cop::registry::CopRegistry;

use state::State;

#[cfg(test)]
use state::FixedLint;

/// MCP session: shared lint state + tool router.
#[derive(Clone)]
pub struct RuboCopMcp {
    state: Arc<State>,
    #[allow(dead_code)] // read by `#[tool_handler]` macro glue
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct InspectionArgs {
    /// File/dir string, or array of paths. Walks up for `.rubocop.yml`
    /// (do not rely on the MCP process cwd).
    #[serde(default)]
    path: Option<targets::OneOrMany>,
    /// Multiple files or directories (same as an array in `path`).
    #[serde(default)]
    paths: Vec<String>,
    /// Inline Ruby source (skips filesystem discovery). Pass `path` with it
    /// so Include filters and project config still apply.
    #[serde(default)]
    source_code: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AutocorrectArgs {
    /// File/dir string, or array of paths. Walks up for `.rubocop.yml`.
    #[serde(default)]
    path: Option<targets::OneOrMany>,
    /// Multiple files or directories (same as an array in `path`).
    #[serde(default)]
    paths: Vec<String>,
    /// Inline Ruby source to correct.
    #[serde(default)]
    source_code: Option<String>,
    /// `true` = safe corrections only; `false` includes unsafe.
    safety: bool,
}

#[tool_router]
impl RuboCopMcp {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: Arc::new(State {
                registry: CopRegistry::default_registry(),
                fixed: None,
            }),
            tool_router: Self::tool_router(),
        })
    }

    /// Built-in defaults only (ignores project / home `.rubocop.yml`). For tests.
    #[cfg(test)]
    fn with_defaults() -> Self {
        use crate::config::{load_default_config, CopFilterSet};

        let registry = CopRegistry::default_registry();
        let config = load_default_config(None, None);
        Self {
            state: Arc::new(State {
                fixed: Some(FixedLint {
                    filters: CopFilterSet::build(&config, &registry),
                    config,
                }),
                registry,
            }),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "rubocop_inspection",
        description = "Inspect Ruby code for offenses. Pass `path` (string or array) and/or `paths`, preferably absolute, so the nearest `.rubocop.yml` is used. Optionally pass `source_code` for unsaved buffers together with `path`.",
        annotations(
            title = "RuboCop's inspection",
            read_only_hint = true,
            idempotent_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    fn inspection(
        &self,
        Parameters(args): Parameters<InspectionArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(tool_result(tools::inspect(
            &self.state,
            targets::merge(args.path, args.paths),
            args.source_code,
        )))
    }

    #[tool(
        name = "rubocop_autocorrection",
        description = "Autocorrect RuboCop offenses. Pass `path` (string or array) and/or `paths`, preferably absolute. Set `safety` to false to include unsafe corrections.",
        annotations(
            title = "RuboCop's autocorrection",
            read_only_hint = false,
            idempotent_hint = false,
            destructive_hint = true,
            open_world_hint = false
        )
    )]
    fn autocorrection(
        &self,
        Parameters(args): Parameters<AutocorrectArgs>,
    ) -> Result<CallToolResult, McpError> {
        Ok(tool_result(tools::autocorrect(
            &self.state,
            targets::merge(args.path, args.paths),
            args.source_code,
            args.safety,
        )))
    }
}

fn tool_result(r: Result<String, String>) -> CallToolResult {
    match r {
        Ok(text) => CallToolResult::success(vec![ContentBlock::text(text)]),
        Err(text) => CallToolResult::error(vec![ContentBlock::text(text)]),
    }
}

#[tool_handler]
impl ServerHandler for RuboCopMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "rrubocop_mcp_server",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_protocol_version(ProtocolVersion::V_2025_06_18)
            .with_instructions(
                "RuboCop-compatible lint tools: rubocop_inspection, rubocop_autocorrection. \
                 Always pass `path` or `paths` (absolute file/dir; `path` may be a string or \
                 array). Config is resolved by walking up from the first target, not from the \
                 MCP process cwd."
                    .to_string(),
            )
    }
}

/// CLI entry: serve MCP over stdin/stdout until the client disconnects.
pub fn run() -> Result<ExitCode> {
    let rt = Runtime::new()?;
    rt.block_on(async {
        let server = RuboCopMcp::new()?;
        let service = server.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok(ExitCode::SUCCESS)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::CallToolRequestParams;
    use serde_json::Map;

    async fn with_client<F, Fut>(f: F)
    where
        F: FnOnce(rmcp::service::RunningService<rmcp::RoleClient, ()>) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        serve_client(RuboCopMcp::with_defaults(), f).await;
    }

    async fn with_live_client<F, Fut>(f: F)
    where
        F: FnOnce(rmcp::service::RunningService<rmcp::RoleClient, ()>) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        serve_client(RuboCopMcp::new().expect("mcp"), f).await;
    }

    async fn serve_client<F, Fut>(server: RuboCopMcp, f: F)
    where
        F: FnOnce(rmcp::service::RunningService<rmcp::RoleClient, ()>) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let (server_side, client_side) = tokio::io::duplex(64 * 1024);
        let server_task = tokio::spawn(async move {
            let _ = server
                .serve(server_side)
                .await
                .expect("serve")
                .waiting()
                .await;
        });
        f(().serve(client_side).await.expect("client")).await;
        server_task.abort();
    }

    fn args_map(v: serde_json::Value) -> Map<String, serde_json::Value> {
        v.as_object().expect("object").clone()
    }

    fn long_example_spec() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".rubocop.yml"),
            "RSpec/ExampleLength:\n  Enabled: true\n  Max: 3\n",
        )
        .unwrap();
        let spec = dir.path().join("example_length_spec.rb");
        std::fs::write(
            &spec,
            "RSpec.describe('x') do\n  it 'long' do\n    a = 1\n    b = 2\n    c = 3\n    d = 4\n  end\nend\n",
        )
        .unwrap();
        (dir, spec)
    }

    #[tokio::test]
    async fn list_tools() {
        with_client(|client| async move {
            let names: Vec<_> = client
                .list_all_tools()
                .await
                .expect("list")
                .iter()
                .map(|t| t.name.as_ref().to_string())
                .collect();
            assert!(names.iter().any(|n| n == "rubocop_inspection"));
            assert!(names.iter().any(|n| n == "rubocop_autocorrection"));
            assert_eq!(names.len(), 2);
            let _ = client.cancel().await;
        })
        .await;
    }

    #[tokio::test]
    async fn inspect_path_uses_project_config() {
        with_live_client(|client| async move {
            let (_dir, spec) = long_example_spec();
            let result = client
                .call_tool(
                    CallToolRequestParams::new("rubocop_inspection").with_arguments(args_map(
                        serde_json::json!({ "path": spec.to_string_lossy() }),
                    )),
                )
                .await
                .expect("call");
            assert_eq!(result.is_error, Some(false));
            let body = result.content[0].as_text().unwrap().text.as_str();
            assert!(body.contains("RSpec/ExampleLength"), "got: {body}");
            let _ = client.cancel().await;
        })
        .await;
    }

    #[tokio::test]
    async fn inspect_inline_source() {
        with_client(|client| async move {
            let result = client
                .call_tool(
                    CallToolRequestParams::new("rubocop_inspection")
                        .with_arguments(args_map(serde_json::json!({ "source_code": "?a" }))),
                )
                .await
                .expect("call");
            assert_eq!(result.is_error, Some(false));
            let offenses: serde_json::Value =
                serde_json::from_str(result.content[0].as_text().unwrap().text.as_str()).unwrap();
            assert!(offenses
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o["code"] == "Style/CharacterLiteral"));
            let _ = client.cancel().await;
        })
        .await;
    }

    #[tokio::test]
    async fn inspect_without_targets_errors() {
        with_client(|client| async move {
            let result = client
                .call_tool(
                    CallToolRequestParams::new("rubocop_inspection")
                        .with_arguments(args_map(serde_json::json!({}))),
                )
                .await
                .expect("call");
            assert!(result.is_error == Some(true));
            let text = result.content[0].as_text().unwrap().text.clone();
            assert!(text.contains("`path` or `paths`"), "{text}");
            let _ = client.cancel().await;
        })
        .await;
    }

    #[tokio::test]
    async fn inspect_path_array_accepted() {
        with_client(|client| async move {
            let result = client
                .call_tool(
                    CallToolRequestParams::new("rubocop_inspection").with_arguments(args_map(
                        serde_json::json!({ "path": ["/no/such/rrubocop_mcp.rb"] }),
                    )),
                )
                .await
                .expect("call");
            assert!(result.is_error == Some(true));
            let text = result.content[0].as_text().unwrap().text.clone();
            assert!(text.contains("No such file"), "{text}");
            let _ = client.cancel().await;
        })
        .await;
    }

    #[tokio::test]
    async fn inspect_paths_key_accepted() {
        with_client(|client| async move {
            let result = client
                .call_tool(
                    CallToolRequestParams::new("rubocop_inspection").with_arguments(args_map(
                        serde_json::json!({ "paths": ["/no/such/rrubocop_mcp.rb"] }),
                    )),
                )
                .await
                .expect("call");
            assert!(result.is_error == Some(true));
            let text = result.content[0].as_text().unwrap().text.clone();
            assert!(text.contains("No such file"), "{text}");
            let _ = client.cancel().await;
        })
        .await;
    }

    #[tokio::test]
    async fn autocorrect_without_targets_errors() {
        with_client(|client| async move {
            let result = client
                .call_tool(
                    CallToolRequestParams::new("rubocop_autocorrection").with_arguments(args_map(
                        serde_json::json!({ "safety": true }),
                    )),
                )
                .await
                .expect("call");
            assert!(result.is_error == Some(true));
            let text = result.content[0].as_text().unwrap().text.clone();
            assert!(text.contains("`path` or `paths`"), "{text}");
            let _ = client.cancel().await;
        })
        .await;
    }

    #[tokio::test]
    async fn autocorrect_safe_inline() {
        with_client(|client| async move {
            let result = client
                .call_tool(
                    CallToolRequestParams::new("rubocop_autocorrection").with_arguments(args_map(
                        serde_json::json!({ "safety": true, "source_code": "?a" }),
                    )),
                )
                .await
                .expect("call");
            assert_eq!(result.is_error, Some(false));
            assert!(
                result.content[0]
                    .as_text()
                    .unwrap()
                    .text
                    .contains("'a'"),
                "got: {:?}",
                result.content[0].as_text().unwrap().text
            );
            let _ = client.cancel().await;
        })
        .await;
    }
}
