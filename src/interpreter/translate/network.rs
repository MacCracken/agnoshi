use anyhow::{Result, anyhow};

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

/// Validate a network target (hostname, IP, CIDR, URL) for shell safety.
/// Rejects targets containing shell metacharacters.
fn validate_target(target: &str) -> Result<()> {
    let dangerous = [
        ';', '&', '|', '`', '$', '(', ')', '{', '}', '<', '>', '!', '\n', '\r', '\0',
    ];
    if target.chars().any(|c| dangerous.contains(&c)) {
        return Err(anyhow!("Target contains disallowed characters: {}", target));
    }
    if target.len() > 253 {
        return Err(anyhow!("Target too long (max 253 characters)"));
    }
    Ok(())
}

pub(crate) fn translate_network(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::NetworkScan { action, target } => {
            // Validate target if provided
            if let Some(t) = target.as_deref() {
                validate_target(t)?;
            }
            let (command, args, desc, permission) = match action.as_str() {
                "port_scan" => {
                    let t = target.as_deref().unwrap_or("localhost");
                    (
                        "nmap".to_string(),
                        vec!["-sT".to_string(), t.to_string()],
                        format!("Port scan on {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "ping_sweep" => {
                    let t = target.as_deref().unwrap_or("192.168.1.0/24");
                    (
                        "nmap".to_string(),
                        vec!["-sn".to_string(), t.to_string()],
                        format!("Ping sweep on {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "dns_lookup" => {
                    let t = target.as_deref().unwrap_or("localhost");
                    (
                        "dig".to_string(),
                        vec![t.to_string()],
                        format!("DNS lookup for {}", t),
                        PermissionLevel::Safe,
                    )
                }
                "trace_route" => {
                    let t = target.as_deref().unwrap_or("localhost");
                    (
                        "traceroute".to_string(),
                        vec![t.to_string()],
                        format!("Trace route to {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "packet_capture" => {
                    let iface = target.as_deref().unwrap_or("eth0");
                    (
                        "tcpdump".to_string(),
                        vec![
                            "-i".to_string(),
                            iface.to_string(),
                            "-c".to_string(),
                            "100".to_string(),
                        ],
                        format!("Capture packets on {}", iface),
                        PermissionLevel::Admin,
                    )
                }
                "web_scan" => {
                    let t = target.as_deref().unwrap_or("http://localhost");
                    (
                        "nikto".to_string(),
                        vec!["-h".to_string(), t.to_string()],
                        format!("Web scan on {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "mass_scan" => {
                    let t = target.as_deref().unwrap_or("192.168.1.0/24");
                    (
                        "masscan".to_string(),
                        vec![
                            "--rate=1000".to_string(),
                            "-p1-65535".to_string(),
                            t.to_string(),
                        ],
                        format!("Mass scan on {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "arp_scan" => {
                    let args = if let Some(t) = target.as_deref() {
                        vec![t.to_string()]
                    } else {
                        vec!["--localnet".to_string()]
                    };
                    (
                        "arp-scan".to_string(),
                        args,
                        "ARP scan local network".to_string(),
                        PermissionLevel::Admin,
                    )
                }
                "network_diag" => {
                    let t = target.as_deref().unwrap_or("localhost");
                    (
                        "mtr".to_string(),
                        vec![
                            "--report".to_string(),
                            "-c".to_string(),
                            "10".to_string(),
                            t.to_string(),
                        ],
                        format!("Network diagnostics to {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "service_scan" => {
                    let t = target.as_deref().unwrap_or("localhost");
                    (
                        "nmap".to_string(),
                        vec!["-sV".to_string(), t.to_string()],
                        format!("Service detection on {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "dir_fuzz" => {
                    let t = target.as_deref().unwrap_or("http://localhost");
                    (
                        "ffuf".to_string(),
                        vec![
                            "-u".to_string(),
                            format!("{}/FUZZ", t),
                            "-w".to_string(),
                            "/usr/share/wordlists/common.txt".to_string(),
                        ],
                        format!("Directory fuzzing on {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "vuln_scan" => {
                    let t = target.as_deref().unwrap_or("http://localhost");
                    (
                        "nuclei".to_string(),
                        vec!["-u".to_string(), t.to_string(), "-silent".to_string()],
                        format!("Vulnerability scan on {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "socket_stats" => (
                    "ss".to_string(),
                    vec!["-tunap".to_string()],
                    "Show network sockets and connections".to_string(),
                    PermissionLevel::Safe,
                ),
                "dns_enum" => {
                    let t = target.as_deref().unwrap_or("localhost");
                    (
                        "dnsrecon".to_string(),
                        vec!["-d".to_string(), t.to_string()],
                        format!("DNS enumeration for {}", t),
                        PermissionLevel::Admin,
                    )
                }
                "deep_inspect" => {
                    let iface = target.as_deref().unwrap_or("eth0");
                    (
                        "tshark".to_string(),
                        vec![
                            "-i".to_string(),
                            iface.to_string(),
                            "-c".to_string(),
                            "100".to_string(),
                        ],
                        format!("Deep packet inspection on {}", iface),
                        PermissionLevel::Admin,
                    )
                }
                "bandwidth_monitor" => (
                    "nethogs".to_string(),
                    vec![],
                    "Monitor per-process bandwidth usage".to_string(),
                    PermissionLevel::Admin,
                ),
                other => {
                    return Err(anyhow!("Unknown network scan action: {}", other));
                }
            };
            Ok(Translation {
                command,
                args,
                description: desc.clone(),
                permission,
                explanation: desc,
                mcp: None,
            })
        }

        _ => unreachable!("translate_network called with non-network intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_target_rejects_semicolon() {
        assert!(validate_target("192.168.1.1; rm -rf /").is_err());
    }

    #[test]
    fn test_validate_target_rejects_backtick() {
        assert!(validate_target("`whoami`").is_err());
    }

    #[test]
    fn test_validate_target_rejects_pipe() {
        assert!(validate_target("192.168.1.1 | cat /etc/passwd").is_err());
    }

    #[test]
    fn test_validate_target_rejects_dollar() {
        assert!(validate_target("$(cat /etc/shadow)").is_err());
    }

    #[test]
    fn test_validate_target_rejects_too_long() {
        assert!(validate_target(&"a".repeat(254)).is_err());
    }

    #[test]
    fn test_validate_target_accepts_ip() {
        assert!(validate_target("192.168.1.1").is_ok());
    }

    #[test]
    fn test_validate_target_accepts_cidr() {
        assert!(validate_target("10.0.0.0/24").is_ok());
    }

    #[test]
    fn test_validate_target_accepts_hostname() {
        assert!(validate_target("example.com").is_ok());
    }

    #[test]
    fn test_validate_target_accepts_url() {
        assert!(validate_target("http://localhost:8080/path").is_ok());
    }

    #[test]
    fn test_network_scan_with_injected_target_fails() {
        let intent = Intent::NetworkScan {
            action: "port_scan".to_string(),
            target: Some("localhost; cat /etc/passwd".to_string()),
        };
        assert!(translate_network(&intent).is_err());
    }

    #[test]
    fn test_network_scan_with_valid_target_succeeds() {
        let intent = Intent::NetworkScan {
            action: "port_scan".to_string(),
            target: Some("192.168.1.1".to_string()),
        };
        let result = translate_network(&intent).unwrap();
        assert_eq!(result.command, "nmap");
    }

    #[test]
    fn test_validate_target_rejects_null_byte() {
        assert!(validate_target("192.168.1.1\0; rm -rf /").is_err());
    }

    #[test]
    fn test_validate_target_rejects_null_byte_only() {
        assert!(validate_target("\0").is_err());
    }

    #[test]
    fn test_network_scan_rejects_null_byte_in_target() {
        let intent = Intent::NetworkScan {
            action: "dns_lookup".to_string(),
            target: Some("example.com\0.evil.com".to_string()),
        };
        assert!(translate_network(&intent).is_err());
    }

    #[test]
    fn test_port_scan_default_target() {
        let intent = Intent::NetworkScan {
            action: "port_scan".to_string(),
            target: None,
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "nmap");
        assert!(t.args.contains(&"-sT".to_string()));
        assert!(t.args.contains(&"localhost".to_string()));
        assert_eq!(t.permission, PermissionLevel::Admin);
    }

    #[test]
    fn test_ping_sweep() {
        let intent = Intent::NetworkScan {
            action: "ping_sweep".to_string(),
            target: Some("10.0.0.0/24".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "nmap");
        assert!(t.args.contains(&"-sn".to_string()));
        assert!(t.args.contains(&"10.0.0.0/24".to_string()));
    }

    #[test]
    fn test_ping_sweep_default() {
        let intent = Intent::NetworkScan {
            action: "ping_sweep".to_string(),
            target: None,
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "nmap");
        assert!(t.args.contains(&"192.168.1.0/24".to_string()));
    }

    #[test]
    fn test_dns_lookup() {
        let intent = Intent::NetworkScan {
            action: "dns_lookup".to_string(),
            target: Some("example.com".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "dig");
        assert!(t.args.contains(&"example.com".to_string()));
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_trace_route() {
        let intent = Intent::NetworkScan {
            action: "trace_route".to_string(),
            target: Some("8.8.8.8".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "traceroute");
        assert!(t.args.contains(&"8.8.8.8".to_string()));
        assert_eq!(t.permission, PermissionLevel::Admin);
    }

    #[test]
    fn test_packet_capture() {
        let intent = Intent::NetworkScan {
            action: "packet_capture".to_string(),
            target: Some("wlan0".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "tcpdump");
        assert!(t.args.contains(&"-i".to_string()));
        assert!(t.args.contains(&"wlan0".to_string()));
        assert!(t.args.contains(&"-c".to_string()));
        assert!(t.args.contains(&"100".to_string()));
    }

    #[test]
    fn test_packet_capture_default() {
        let intent = Intent::NetworkScan {
            action: "packet_capture".to_string(),
            target: None,
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "tcpdump");
        assert!(t.args.contains(&"eth0".to_string()));
    }

    #[test]
    fn test_web_scan() {
        let intent = Intent::NetworkScan {
            action: "web_scan".to_string(),
            target: Some("http://example.com".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "nikto");
        assert!(t.args.contains(&"-h".to_string()));
        assert!(t.args.contains(&"http://example.com".to_string()));
    }

    #[test]
    fn test_mass_scan() {
        let intent = Intent::NetworkScan {
            action: "mass_scan".to_string(),
            target: Some("10.0.0.0/8".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "masscan");
        assert!(t.args.contains(&"--rate=1000".to_string()));
        assert!(t.args.contains(&"-p1-65535".to_string()));
        assert!(t.args.contains(&"10.0.0.0/8".to_string()));
    }

    #[test]
    fn test_arp_scan_with_target() {
        let intent = Intent::NetworkScan {
            action: "arp_scan".to_string(),
            target: Some("192.168.1.0/24".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "arp-scan");
        assert!(t.args.contains(&"192.168.1.0/24".to_string()));
    }

    #[test]
    fn test_arp_scan_default() {
        let intent = Intent::NetworkScan {
            action: "arp_scan".to_string(),
            target: None,
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "arp-scan");
        assert!(t.args.contains(&"--localnet".to_string()));
    }

    #[test]
    fn test_network_diag() {
        let intent = Intent::NetworkScan {
            action: "network_diag".to_string(),
            target: Some("google.com".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "mtr");
        assert!(t.args.contains(&"--report".to_string()));
        assert!(t.args.contains(&"-c".to_string()));
        assert!(t.args.contains(&"10".to_string()));
        assert!(t.args.contains(&"google.com".to_string()));
    }

    #[test]
    fn test_service_scan() {
        let intent = Intent::NetworkScan {
            action: "service_scan".to_string(),
            target: Some("192.168.1.1".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "nmap");
        assert!(t.args.contains(&"-sV".to_string()));
        assert!(t.args.contains(&"192.168.1.1".to_string()));
    }

    #[test]
    fn test_dir_fuzz() {
        let intent = Intent::NetworkScan {
            action: "dir_fuzz".to_string(),
            target: Some("http://target.local".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "ffuf");
        assert!(t.args.contains(&"-u".to_string()));
        assert!(t.args.contains(&"http://target.local/FUZZ".to_string()));
        assert!(t.args.contains(&"-w".to_string()));
    }

    #[test]
    fn test_vuln_scan() {
        let intent = Intent::NetworkScan {
            action: "vuln_scan".to_string(),
            target: Some("http://app.local".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "nuclei");
        assert!(t.args.contains(&"-u".to_string()));
        assert!(t.args.contains(&"http://app.local".to_string()));
        assert!(t.args.contains(&"-silent".to_string()));
    }

    #[test]
    fn test_socket_stats() {
        let intent = Intent::NetworkScan {
            action: "socket_stats".to_string(),
            target: None,
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "ss");
        assert!(t.args.contains(&"-tunap".to_string()));
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_dns_enum() {
        let intent = Intent::NetworkScan {
            action: "dns_enum".to_string(),
            target: Some("example.org".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "dnsrecon");
        assert!(t.args.contains(&"-d".to_string()));
        assert!(t.args.contains(&"example.org".to_string()));
    }

    #[test]
    fn test_deep_inspect() {
        let intent = Intent::NetworkScan {
            action: "deep_inspect".to_string(),
            target: Some("br0".to_string()),
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "tshark");
        assert!(t.args.contains(&"-i".to_string()));
        assert!(t.args.contains(&"br0".to_string()));
        assert!(t.args.contains(&"-c".to_string()));
        assert!(t.args.contains(&"100".to_string()));
    }

    #[test]
    fn test_bandwidth_monitor() {
        let intent = Intent::NetworkScan {
            action: "bandwidth_monitor".to_string(),
            target: None,
        };
        let t = translate_network(&intent).unwrap();
        assert_eq!(t.command, "nethogs");
        assert!(t.args.is_empty());
        assert_eq!(t.permission, PermissionLevel::Admin);
    }

    #[test]
    fn test_unknown_action_returns_error() {
        let intent = Intent::NetworkScan {
            action: "nonexistent".to_string(),
            target: None,
        };
        assert!(translate_network(&intent).is_err());
    }
}
