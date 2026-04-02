use anyhow::{Result, anyhow};

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

/// Translate t-ron security monitor intents to MCP tool calls.
pub(crate) fn translate_tron(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::TronStatus => {
            let args = serde_json::Map::new();
            Ok(mcp_call(
                "tron_status",
                args,
                "Get t-ron security status".to_string(),
                PermissionLevel::Safe,
                "Queries t-ron for overall security status: events, denials, health".to_string(),
            ))
        }

        Intent::TronRisk { agent_id } => {
            let mut args = serde_json::Map::new();
            insert_str(&mut args, "agent_id", agent_id);
            Ok(mcp_call(
                "tron_risk",
                args,
                format!("Get risk score for agent '{agent_id}'"),
                PermissionLevel::Safe,
                "Queries t-ron for per-agent risk score (0.0 trusted to 1.0 hostile)".to_string(),
            ))
        }

        Intent::TronAudit { agent_id, limit } => {
            let mut args = serde_json::Map::new();
            insert_opt(&mut args, "agent_id", agent_id);
            if let Some(n) = limit {
                args.insert(
                    "limit".to_string(),
                    serde_json::Value::Number((*n as u64).into()),
                );
            }
            let desc = match agent_id {
                Some(id) => format!("View security audit events for agent '{id}'"),
                None => "View recent security audit events".to_string(),
            };
            Ok(mcp_call(
                "tron_audit",
                args,
                desc,
                PermissionLevel::Safe,
                "Retrieves recent security events from t-ron audit log".to_string(),
            ))
        }

        Intent::TronPolicy { toml } => {
            let mut args = serde_json::Map::new();
            insert_str(&mut args, "toml", toml);
            Ok(mcp_call(
                "tron_policy",
                args,
                "Load security policy".to_string(),
                PermissionLevel::Admin,
                "Loads or reloads t-ron security policy from TOML content".to_string(),
            ))
        }

        _ => Err(anyhow!("translate_tron called with non-tron intent")),
    }
}
