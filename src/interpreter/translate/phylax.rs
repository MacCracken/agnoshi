use anyhow::{Result, anyhow};

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_phylax(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::PhylaxScan { target, mode } => {
            let mode_str = mode.as_deref().unwrap_or("on_demand");
            Ok(Translation {
                command: "curl".to_string(),
                args: vec![
                    "-s".to_string(),
                    "-X".to_string(),
                    "POST".to_string(),
                    "http://127.0.0.1:8090/v1/scan/file".to_string(),
                    "-H".to_string(),
                    "Content-Type: application/json".to_string(),
                    "-d".to_string(),
                    format!(r#"{{"path":"{}","mode":"{}"}}"#, target, mode_str),
                ],
                description: format!("Scan {} for threats", target),
                permission: PermissionLevel::Admin,
                explanation: "Runs phylax threat detection engine on target file".to_string(),
                mcp: None,
            })
        }
        Intent::PhylaxFindings { severity } => {
            let query = severity
                .as_ref()
                .map(|s| format!("?severity={}", s))
                .unwrap_or_default();
            Ok(Translation {
                command: "curl".to_string(),
                args: vec![
                    "-s".to_string(),
                    format!("http://127.0.0.1:8090/v1/scan/history{}", query),
                ],
                description: "View threat scan findings".to_string(),
                permission: PermissionLevel::ReadOnly,
                explanation: "Retrieves phylax threat detection findings".to_string(),
                mcp: None,
            })
        }
        Intent::PhylaxHistory { .. } => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "http://127.0.0.1:8090/v1/scan/history".to_string(),
            ],
            description: "View phylax scan history".to_string(),
            permission: PermissionLevel::ReadOnly,
            explanation: "Lists recent phylax scan results".to_string(),
            mcp: None,
        }),
        Intent::PhylaxStatus => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "http://127.0.0.1:8090/v1/scan/status".to_string(),
            ],
            description: "Get phylax scanner status".to_string(),
            permission: PermissionLevel::ReadOnly,
            explanation: "Shows phylax threat detection engine statistics".to_string(),
            mcp: None,
        }),
        Intent::PhylaxRules => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "http://127.0.0.1:8090/v1/scan/rules".to_string(),
            ],
            description: "List phylax detection rules".to_string(),
            permission: PermissionLevel::ReadOnly,
            explanation: "Lists all loaded YARA-compatible detection rules".to_string(),
            mcp: None,
        }),
        _ => Err(anyhow!("translate_phylax called with non-phylax intent")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phylax_scan_default_mode() {
        let intent = Intent::PhylaxScan {
            target: "/tmp/suspect.bin".to_string(),
            mode: None,
        };
        let t = translate_phylax(&intent).unwrap();
        assert_eq!(t.command, "curl");
        assert!(t.args.iter().any(|a| a.contains("on_demand")));
        assert!(t.args.iter().any(|a| a.contains("/tmp/suspect.bin")));
        assert_eq!(t.permission, PermissionLevel::Admin);
    }

    #[test]
    fn test_phylax_scan_custom_mode() {
        let intent = Intent::PhylaxScan {
            target: "/usr/bin/app".to_string(),
            mode: Some("pre_exec".to_string()),
        };
        let t = translate_phylax(&intent).unwrap();
        assert_eq!(t.command, "curl");
        assert!(t.args.iter().any(|a| a.contains("pre_exec")));
        assert!(t.args.iter().any(|a| a.contains("/usr/bin/app")));
    }

    #[test]
    fn test_phylax_findings_no_filter() {
        let intent = Intent::PhylaxFindings { severity: None };
        let t = translate_phylax(&intent).unwrap();
        assert_eq!(t.command, "curl");
        assert!(
            t.args
                .iter()
                .any(|a| a == "http://127.0.0.1:8090/v1/scan/history")
        );
        assert_eq!(t.permission, PermissionLevel::ReadOnly);
    }

    #[test]
    fn test_phylax_findings_with_severity() {
        let intent = Intent::PhylaxFindings {
            severity: Some("critical".to_string()),
        };
        let t = translate_phylax(&intent).unwrap();
        assert_eq!(t.command, "curl");
        assert!(t.args.iter().any(|a| a.contains("severity=critical")));
    }

    #[test]
    fn test_phylax_history() {
        let intent = Intent::PhylaxHistory { limit: Some(10) };
        let t = translate_phylax(&intent).unwrap();
        assert_eq!(t.command, "curl");
        assert!(
            t.args
                .iter()
                .any(|a| a == "http://127.0.0.1:8090/v1/scan/history")
        );
        assert_eq!(t.permission, PermissionLevel::ReadOnly);
    }

    #[test]
    fn test_phylax_status() {
        let intent = Intent::PhylaxStatus;
        let t = translate_phylax(&intent).unwrap();
        assert_eq!(t.command, "curl");
        assert!(
            t.args
                .iter()
                .any(|a| a == "http://127.0.0.1:8090/v1/scan/status")
        );
        assert_eq!(t.permission, PermissionLevel::ReadOnly);
    }

    #[test]
    fn test_phylax_rules() {
        let intent = Intent::PhylaxRules;
        let t = translate_phylax(&intent).unwrap();
        assert_eq!(t.command, "curl");
        assert!(
            t.args
                .iter()
                .any(|a| a == "http://127.0.0.1:8090/v1/scan/rules")
        );
        assert_eq!(t.permission, PermissionLevel::ReadOnly);
    }

    #[test]
    fn test_phylax_non_phylax_intent_errors() {
        let intent = Intent::PhylaxStatus; // valid
        assert!(translate_phylax(&intent).is_ok());

        let intent = Intent::SystemInfo;
        assert!(translate_phylax(&intent).is_err());
    }
}
