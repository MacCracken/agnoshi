use anyhow::{Result, bail};
use serde_json::json;

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

/// Reject values containing path traversal sequences or URL-unsafe characters.
fn sanitize_url_segment(value: &str) -> Result<&str> {
    if value.contains("..") || value.contains('/') || value.contains('\\') || value.contains('\0') {
        bail!("Invalid identifier: contains path traversal characters");
    }
    Ok(value)
}

pub(crate) fn translate_marketplace(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::MarketplaceInstall { package } => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "-X".to_string(),
                "POST".to_string(),
                "http://127.0.0.1:8090/v1/marketplace/install".to_string(),
                "-H".to_string(),
                "Content-Type: application/json".to_string(),
                "-d".to_string(),
                json!({"path": package}).to_string(),
            ],
            description: format!("Install marketplace package: {}", package),
            permission: PermissionLevel::SystemWrite,
            explanation: "Installs a package from the marketplace".to_string(),
            mcp: None,
        }),

        Intent::MarketplaceUninstall { package } => {
            let safe_pkg = sanitize_url_segment(package)?;
            Ok(Translation {
                command: "curl".to_string(),
                args: vec![
                    "-s".to_string(),
                    "-X".to_string(),
                    "DELETE".to_string(),
                    format!("http://127.0.0.1:8090/v1/marketplace/{}", safe_pkg),
                ],
                description: format!("Uninstall marketplace package: {}", package),
                permission: PermissionLevel::SystemWrite,
                explanation: "Removes an installed marketplace package".to_string(),
                mcp: None,
            })
        }

        Intent::MarketplaceSearch { query } => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "-X".to_string(),
                "POST".to_string(),
                "http://127.0.0.1:8090/v1/marketplace/search".to_string(),
                "-H".to_string(),
                "Content-Type: application/json".to_string(),
                "-d".to_string(),
                json!({"q": query}).to_string(),
            ],
            description: format!("Search marketplace for: {}", query),
            permission: PermissionLevel::Safe,
            explanation: "Searches installed marketplace packages".to_string(),
            mcp: None,
        }),

        Intent::MarketplaceList => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "http://127.0.0.1:8090/v1/marketplace/installed".to_string(),
            ],
            description: "List installed marketplace packages".to_string(),
            permission: PermissionLevel::Safe,
            explanation: "Shows all packages installed from the marketplace".to_string(),
            mcp: None,
        }),

        Intent::MarketplaceUpdate => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "http://127.0.0.1:8090/v1/marketplace/installed".to_string(),
            ],
            description: "Check for marketplace package updates".to_string(),
            permission: PermissionLevel::Safe,
            explanation: "Checks for available updates to installed packages".to_string(),
            mcp: None,
        }),

        _ => unreachable!("translate_marketplace called with non-marketplace intent"),
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
    fn test_sanitize_url_segment_accepts_name_with_single_dots() {
        // Single dots are fine, only ".." is rejected
        assert_eq!(sanitize_url_segment("v1.2.3").unwrap(), "v1.2.3");
    }

    #[test]
    fn test_sanitize_url_segment_rejects_double_dot() {
        assert!(sanitize_url_segment("foo..bar").is_err());
    }

    #[test]
    fn test_marketplace_uninstall_rejects_path_traversal() {
        let intent = Intent::MarketplaceUninstall {
            package: "../../../etc/passwd".to_string(),
        };
        assert!(translate_marketplace(&intent).is_err());
    }

    #[test]
    fn test_marketplace_uninstall_rejects_null_byte() {
        let intent = Intent::MarketplaceUninstall {
            package: "package\0name".to_string(),
        };
        assert!(translate_marketplace(&intent).is_err());
    }

    #[test]
    fn test_marketplace_uninstall_accepts_normal_package() {
        let intent = Intent::MarketplaceUninstall {
            package: "my-cool-package".to_string(),
        };
        let result = translate_marketplace(&intent).unwrap();
        assert_eq!(result.command, "curl");
        assert!(result.args.iter().any(|a| a.contains("my-cool-package")));
    }
}
