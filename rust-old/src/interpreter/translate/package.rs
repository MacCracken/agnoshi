use anyhow::{Result, bail};
use serde_json::json;

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

/// Reject values containing path traversal sequences or URL-unsafe characters.
fn sanitize_url_segment(value: &str) -> Result<&str> {
    if value.contains("..")
        || value.contains('/')
        || value.contains('\\')
        || value.contains('\0')
        || value.contains('?')
        || value.contains('&')
        || value.contains('#')
        || value.contains('%')
        || value.contains('=')
    {
        bail!("Invalid identifier: contains path traversal or URL-special characters");
    }
    Ok(value)
}

pub(crate) fn translate_package(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::InstallPackage { packages } => {
            let mut args = vec!["install".to_string(), "-y".to_string()];
            args.extend(packages.iter().cloned());
            Ok(Translation {
                command: "apt-get".to_string(),
                args,
                description: format!("Install package(s): {}", packages.join(", ")),
                permission: PermissionLevel::SystemWrite,
                explanation: "Installs system packages (requires root)".to_string(),
                mcp: None,
            })
        }

        Intent::ArkInstall {
            packages,
            source: _,
        } => {
            let body = serde_json::json!({"packages": packages});
            Ok(Translation {
                command: "curl".to_string(),
                args: vec![
                    "-s".to_string(),
                    "-X".to_string(),
                    "POST".to_string(),
                    "http://127.0.0.1:8090/v1/ark/install".to_string(),
                    "-H".to_string(),
                    "Content-Type: application/json".to_string(),
                    "-d".to_string(),
                    serde_json::to_string(&body).unwrap(),
                ],
                description: format!("Install packages via ark: {}", packages.join(", ")),
                permission: PermissionLevel::SystemWrite,
                explanation: "Installs packages using the AGNOS unified package manager"
                    .to_string(),
                mcp: None,
            })
        }

        Intent::ArkRemove { packages } => {
            let body = serde_json::json!({"packages": packages});
            Ok(Translation {
                command: "curl".to_string(),
                args: vec![
                    "-s".to_string(),
                    "-X".to_string(),
                    "POST".to_string(),
                    "http://127.0.0.1:8090/v1/ark/remove".to_string(),
                    "-H".to_string(),
                    "Content-Type: application/json".to_string(),
                    "-d".to_string(),
                    serde_json::to_string(&body).unwrap(),
                ],
                description: format!("Remove packages via ark: {}", packages.join(", ")),
                permission: PermissionLevel::SystemWrite,
                explanation: "Removes packages using the AGNOS unified package manager".to_string(),
                mcp: None,
            })
        }

        Intent::ArkSearch { query } => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "-X".to_string(),
                "POST".to_string(),
                "http://127.0.0.1:8090/v1/ark/search".to_string(),
                "-H".to_string(),
                "Content-Type: application/json".to_string(),
                "-d".to_string(),
                json!({"q": query}).to_string(),
            ],
            description: format!("Search packages via ark: {}", query),
            permission: PermissionLevel::Safe,
            explanation: "Searches for packages across all configured sources".to_string(),
            mcp: None,
        }),

        Intent::ArkInfo { package } => {
            let safe_pkg = sanitize_url_segment(package)?;
            Ok(Translation {
                command: "curl".to_string(),
                args: vec![
                    "-s".to_string(),
                    format!("http://127.0.0.1:8090/v1/ark/info/{}", safe_pkg),
                ],
                description: format!("Show ark package info: {}", package),
                permission: PermissionLevel::Safe,
                explanation: "Retrieves detailed information about a package".to_string(),
                mcp: None,
            })
        }

        Intent::ArkUpdate => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "-X".to_string(),
                "POST".to_string(),
                "http://127.0.0.1:8090/v1/ark/update".to_string(),
            ],
            description: "Check for package updates via ark".to_string(),
            permission: PermissionLevel::Safe,
            explanation: "Refreshes package index from all configured sources".to_string(),
            mcp: None,
        }),

        Intent::ArkUpgrade { packages } => {
            let body = serde_json::json!({"packages": packages});
            Ok(Translation {
                command: "curl".to_string(),
                args: vec![
                    "-s".to_string(),
                    "-X".to_string(),
                    "POST".to_string(),
                    "http://127.0.0.1:8090/v1/ark/upgrade".to_string(),
                    "-H".to_string(),
                    "Content-Type: application/json".to_string(),
                    "-d".to_string(),
                    serde_json::to_string(&body).unwrap(),
                ],
                description: format!(
                    "Upgrade packages via ark{}",
                    packages
                        .as_ref()
                        .map_or(String::new(), |p| format!(": {}", p.join(", ")))
                ),
                permission: PermissionLevel::SystemWrite,
                explanation: "Upgrades packages to latest versions".to_string(),
                mcp: None,
            })
        }

        Intent::ArkStatus => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "http://127.0.0.1:8090/v1/ark/status".to_string(),
            ],
            description: "Show ark package manager status".to_string(),
            permission: PermissionLevel::Safe,
            explanation: "Displays the status of the AGNOS unified package manager".to_string(),
            mcp: None,
        }),

        _ => unreachable!("translate_package called with non-package intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_url_segment_rejects_path_traversal() {
        assert!(sanitize_url_segment("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_sanitize_url_segment_rejects_forward_slash() {
        assert!(sanitize_url_segment("foo/bar").is_err());
    }

    #[test]
    fn test_sanitize_url_segment_rejects_backslash() {
        assert!(sanitize_url_segment("foo\\bar").is_err());
    }

    #[test]
    fn test_sanitize_url_segment_rejects_null_byte() {
        assert!(sanitize_url_segment("package\0name").is_err());
    }

    #[test]
    fn test_sanitize_url_segment_accepts_normal_name() {
        assert_eq!(sanitize_url_segment("my-package").unwrap(), "my-package");
    }

    #[test]
    fn test_ark_info_rejects_path_traversal() {
        let intent = Intent::ArkInfo {
            package: "../../../etc/passwd".to_string(),
        };
        assert!(translate_package(&intent).is_err());
    }

    #[test]
    fn test_ark_info_rejects_null_byte() {
        let intent = Intent::ArkInfo {
            package: "package\0name".to_string(),
        };
        assert!(translate_package(&intent).is_err());
    }

    #[test]
    fn test_sanitize_url_segment_rejects_question_mark() {
        assert!(sanitize_url_segment("pkg?foo=bar").is_err());
    }

    #[test]
    fn test_sanitize_url_segment_rejects_ampersand() {
        assert!(sanitize_url_segment("pkg&evil=true").is_err());
    }

    #[test]
    fn test_sanitize_url_segment_rejects_hash() {
        assert!(sanitize_url_segment("pkg#fragment").is_err());
    }

    #[test]
    fn test_sanitize_url_segment_rejects_percent_encoding() {
        assert!(sanitize_url_segment("pkg%2fpasswd").is_err());
    }

    #[test]
    fn test_sanitize_url_segment_rejects_equals() {
        assert!(sanitize_url_segment("pkg=value").is_err());
    }

    #[test]
    fn test_ark_info_accepts_normal_package() {
        let intent = Intent::ArkInfo {
            package: "nginx".to_string(),
        };
        let result = translate_package(&intent).unwrap();
        assert_eq!(result.command, "curl");
        assert!(result.args.iter().any(|a| a.contains("nginx")));
    }
}
