use crate::interpreter::intent::{McpCall, Translation};
use crate::security::PermissionLevel;

/// Build a Translation that calls an MCP tool via bote's JSON-RPC protocol.
///
/// The translation carries an `McpCall` with the tool name and arguments.
/// The session executor dispatches this natively via reqwest + bote types
/// instead of shelling out to curl.
pub(crate) fn mcp_call(
    tool_name: &str,
    args: serde_json::Map<String, serde_json::Value>,
    description: String,
    permission: PermissionLevel,
    explanation: String,
) -> Translation {
    Translation {
        command: String::new(),
        args: Vec::new(),
        description,
        permission,
        explanation,
        mcp: Some(McpCall {
            tool_name: tool_name.to_string(),
            arguments: serde_json::Value::Object(args),
        }),
    }
}

/// Convenience: insert an optional string into an args map.
#[inline]
pub(crate) fn insert_opt(
    args: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &Option<String>,
) {
    if let Some(v) = value {
        args.insert(key.to_string(), serde_json::Value::String(v.clone()));
    }
}

/// Convenience: insert a required string into an args map.
#[inline]
pub(crate) fn insert_str(
    args: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &str,
) {
    args.insert(
        key.to_string(),
        serde_json::Value::String(value.to_string()),
    );
}
