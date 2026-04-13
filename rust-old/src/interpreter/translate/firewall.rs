use anyhow::Result;

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_firewall(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::FirewallAllow { port, protocol } => {
            let spec = protocol
                .as_ref()
                .map(|p| format!("{port}/{p}"))
                .unwrap_or_else(|| port.clone());
            Ok(Translation::cmd(
                "sudo",
                vec!["ufw".to_string(), "allow".to_string(), spec.clone()],
                format!("Allow traffic on {spec}"),
                PermissionLevel::Admin,
                "Adds a firewall rule to allow incoming traffic on the specified port",
            ))
        }

        Intent::FirewallDeny { port, protocol } => {
            let spec = protocol
                .as_ref()
                .map(|p| format!("{port}/{p}"))
                .unwrap_or_else(|| port.clone());
            Ok(Translation::cmd(
                "sudo",
                vec!["ufw".to_string(), "deny".to_string(), spec.clone()],
                format!("Deny traffic on {spec}"),
                PermissionLevel::Admin,
                "Adds a firewall rule to deny incoming traffic on the specified port",
            ))
        }

        Intent::FirewallList => Ok(Translation::cmd(
            "sudo",
            vec![
                "ufw".to_string(),
                "status".to_string(),
                "numbered".to_string(),
            ],
            "List firewall rules",
            PermissionLevel::Admin,
            "Displays all active firewall rules with rule numbers",
        )),

        Intent::FirewallStatus => Ok(Translation::cmd(
            "sudo",
            vec![
                "ufw".to_string(),
                "status".to_string(),
                "verbose".to_string(),
            ],
            "Show firewall status",
            PermissionLevel::Admin,
            "Displays whether the firewall is active and its default policies",
        )),

        Intent::FirewallEnable => Ok(Translation::cmd(
            "sudo",
            vec!["ufw".to_string(), "enable".to_string()],
            "Enable firewall",
            PermissionLevel::Admin,
            "Activates the Uncomplicated Firewall",
        )),

        Intent::FirewallDisable => Ok(Translation::cmd(
            "sudo",
            vec!["ufw".to_string(), "disable".to_string()],
            "Disable firewall",
            PermissionLevel::Admin,
            "Deactivates the Uncomplicated Firewall",
        )),

        Intent::FirewallDeleteRule { rule } => Ok(Translation::cmd(
            "sudo",
            vec!["ufw".to_string(), "delete".to_string(), rule.clone()],
            format!("Delete firewall rule: {rule}"),
            PermissionLevel::Admin,
            "Removes a firewall rule by number or specification",
        )),

        _ => unreachable!(),
    }
}
