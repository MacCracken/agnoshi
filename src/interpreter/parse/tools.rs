use crate::interpreter::Interpreter;
use crate::interpreter::intent::Intent;

/// Parse trading + network tool intents: BullShift, network scanning
pub(super) fn parse_tools(interp: &Interpreter, input_lower: &str) -> Option<Intent> {
    // --- BullShift trading intents ---
    if let Some(caps) = interp.try_captures("bullshift_portfolio", input_lower) {
        let action = caps
            .get(1)
            .map_or("summary", |m| m.as_str())
            .trim()
            .to_string();
        let period = caps
            .get(4)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        return Some(Intent::BullShiftPortfolio { action, period });
    }

    if let Some(caps) = interp.try_captures("bullshift_orders", input_lower) {
        let action = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
        let symbol = caps
            .get(4)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        let side = caps
            .get(6)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        if !action.is_empty() {
            return Some(Intent::BullShiftOrders {
                action,
                symbol,
                side,
            });
        }
    }

    if let Some(caps) = interp.try_captures("bullshift_market", input_lower) {
        let action = caps
            .get(1)
            .map_or("quote", |m| m.as_str())
            .trim()
            .to_string();
        let symbol = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        return Some(Intent::BullShiftMarket { action, symbol });
    }

    if let Some(caps) = interp.try_captures("bullshift_alerts", input_lower) {
        let action = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
        let symbol = caps
            .get(4)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        if !action.is_empty() {
            return Some(Intent::BullShiftAlerts { action, symbol });
        }
    }

    if let Some(caps) = interp.try_captures("bullshift_strategy", input_lower) {
        let action = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
        let name = caps
            .get(4)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        if !action.is_empty() {
            return Some(Intent::BullShiftStrategy { action, name });
        }
    }

    if let Some(caps) = interp.try_captures("bullshift_accounts", input_lower) {
        let action = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
        let broker = caps
            .get(4)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        if !action.is_empty() {
            return Some(Intent::BullShiftAccounts { action, broker });
        }
    }

    if let Some(caps) = interp.try_captures("bullshift_history", input_lower) {
        let action = caps
            .get(1)
            .map_or("trades", |m| m.as_str())
            .trim()
            .to_string();
        let period = caps
            .get(4)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        return Some(Intent::BullShiftHistory { action, period });
    }

    // --- Network scanning intents ---
    if let Some(caps) = interp.try_captures("network_scan", input_lower) {
        if let Some(target) = caps.get(2) {
            return Some(Intent::NetworkScan {
                action: "port_scan".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(3) {
            return Some(Intent::NetworkScan {
                action: "ping_sweep".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(4) {
            return Some(Intent::NetworkScan {
                action: "dns_lookup".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(5) {
            return Some(Intent::NetworkScan {
                action: "trace_route".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(6) {
            return Some(Intent::NetworkScan {
                action: "packet_capture".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(7) {
            return Some(Intent::NetworkScan {
                action: "web_scan".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
    }

    // Extended network tool patterns
    if let Some(caps) = interp.try_captures("network_extended", input_lower) {
        let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        if let Some(target) = caps.get(2) {
            return Some(Intent::NetworkScan {
                action: "mass_scan".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if caps.get(3).is_some() || full.contains("arp scan") {
            let target = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty());
            return Some(Intent::NetworkScan {
                action: "arp_scan".into(),
                target,
            });
        }
        if let Some(target) = caps.get(4) {
            return Some(Intent::NetworkScan {
                action: "network_diag".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(5) {
            return Some(Intent::NetworkScan {
                action: "service_scan".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(6) {
            return Some(Intent::NetworkScan {
                action: "dir_fuzz".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(7) {
            return Some(Intent::NetworkScan {
                action: "vuln_scan".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if full.contains("socket") || full.contains("connection") {
            return Some(Intent::NetworkScan {
                action: "socket_stats".into(),
                target: None,
            });
        }
        if let Some(target) = caps.get(8) {
            return Some(Intent::NetworkScan {
                action: "dns_enum".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if let Some(target) = caps.get(9) {
            return Some(Intent::NetworkScan {
                action: "deep_inspect".into(),
                target: Some(target.as_str().trim().to_string()),
            });
        }
        if full.contains("bandwidth") {
            return Some(Intent::NetworkScan {
                action: "bandwidth_monitor".into(),
                target: None,
            });
        }
    }

    // --- Stiva container runtime intents ---
    if let Some(caps) = interp.try_captures("stiva_run", input_lower) {
        let image = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let name = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        let args: Vec<String> = caps
            .get(3)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| s.split_whitespace().map(|w| w.to_string()).collect())
            .unwrap_or_default();
        return Some(Intent::StivaRun { image, name, args });
    }

    if let Some(caps) = interp.try_captures("stiva_stop", input_lower) {
        let id = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        return Some(Intent::StivaStop { id });
    }

    if let Some(caps) = interp.try_captures("stiva_ps", input_lower) {
        let all = caps.get(1).is_some();
        return Some(Intent::StivaPs { all });
    }

    if let Some(caps) = interp.try_captures("stiva_rm", input_lower) {
        let force = caps.get(1).is_some();
        let id = caps
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        return Some(Intent::StivaRm { id, force });
    }

    if let Some(caps) = interp.try_captures("stiva_pull", input_lower) {
        let image = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        return Some(Intent::StivaPull { image });
    }

    if interp.try_captures("stiva_images", input_lower).is_some() {
        return Some(Intent::StivaImages);
    }

    if let Some(caps) = interp.try_captures("stiva_rmi", input_lower) {
        let image = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        return Some(Intent::StivaRmi { image });
    }

    if let Some(caps) = interp.try_captures("stiva_build", input_lower) {
        let path = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let tag = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        return Some(Intent::StivaBuild { path, tag });
    }

    if let Some(caps) = interp.try_captures("stiva_logs", input_lower) {
        let id = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let tail = caps.get(2).and_then(|m| m.as_str().parse::<usize>().ok());
        return Some(Intent::StivaLogs { id, tail });
    }

    if let Some(caps) = interp.try_captures("stiva_exec", input_lower) {
        let id = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let rest = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let mut parts = rest.split_whitespace();
        let command = parts.next().unwrap_or_default().to_string();
        let args: Vec<String> = parts.map(|w| w.to_string()).collect();
        return Some(Intent::StivaExec { id, command, args });
    }

    if let Some(caps) = interp.try_captures("stiva_inspect", input_lower) {
        let target = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        return Some(Intent::StivaInspect { target });
    }

    if let Some(caps) = interp.try_captures("stiva_ansamblu", input_lower) {
        let action = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let file = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty());
        return Some(Intent::StivaAnsamblu { action, file });
    }

    None
}
