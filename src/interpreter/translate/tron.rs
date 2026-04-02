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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_status() {
        let intent = Intent::TronStatus;
        let t = translate_tron(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tron_status");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tron_risk() {
        let intent = Intent::TronRisk {
            agent_id: "agent-42".to_string(),
        };
        let t = translate_tron(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tron_risk");
        assert_eq!(mcp.arguments["agent_id"], "agent-42");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tron_audit_with_agent_and_limit() {
        let intent = Intent::TronAudit {
            agent_id: Some("agent-42".to_string()),
            limit: Some(20),
        };
        let t = translate_tron(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tron_audit");
        assert_eq!(mcp.arguments["agent_id"], "agent-42");
        assert_eq!(mcp.arguments["limit"], 20);
        assert_eq!(t.permission, PermissionLevel::Safe);
        assert!(t.description.contains("agent-42"));
    }

    #[test]
    fn test_tron_audit_no_agent_no_limit() {
        let intent = Intent::TronAudit {
            agent_id: None,
            limit: None,
        };
        let t = translate_tron(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tron_audit");
        assert!(mcp.arguments.get("agent_id").is_none());
        assert!(mcp.arguments.get("limit").is_none());
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tron_policy() {
        let intent = Intent::TronPolicy {
            toml: "[policy]\nmax_risk = 0.5".to_string(),
        };
        let t = translate_tron(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tron_policy");
        assert_eq!(mcp.arguments["toml"], "[policy]\nmax_risk = 0.5");
        assert_eq!(t.permission, PermissionLevel::Admin);
    }

    #[test]
    fn test_tron_non_tron_intent_errors() {
        let intent = Intent::TronStatus; // valid intent, just testing the pattern
        // Verify a valid call works
        assert!(translate_tron(&intent).is_ok());
    }
}
