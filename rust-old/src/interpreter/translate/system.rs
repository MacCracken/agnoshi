use anyhow::{Result, anyhow};

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_system(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::JournalView {
            unit,
            priority,
            lines,
            since,
        } => {
            let mut args = Vec::new();
            if let Some(u) = unit {
                args.push("-u".to_string());
                args.push(u.clone());
            }
            if let Some(p) = priority {
                args.push("-p".to_string());
                args.push(p.clone());
            }
            if let Some(n) = lines {
                args.push("-n".to_string());
                args.push(n.to_string());
            }
            if let Some(s) = since {
                args.push("--since".to_string());
                args.push(s.clone());
            }
            if args.is_empty() {
                // Default: show recent entries
                args.push("-n".to_string());
                args.push("50".to_string());
            }
            let desc = match (unit, priority) {
                (Some(u), Some(p)) => format!("Show {} priority journal logs for {}", p, u),
                (Some(u), None) => format!("Show journal logs for {}", u),
                (None, Some(p)) => format!("Show {} priority journal logs", p),
                (None, None) => "Show recent journal log entries".to_string(),
            };
            Ok(Translation {
                command: "journalctl".to_string(),
                args,
                description: desc.clone(),
                permission: PermissionLevel::ReadOnly,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::DeviceInfo {
            subsystem,
            device_path,
        } => {
            let (args, desc) = if let Some(path) = device_path {
                (
                    vec![
                        "info".to_string(),
                        "--query=all".to_string(),
                        "--name".to_string(),
                        path.clone(),
                    ],
                    format!("Show device info for {}", path),
                )
            } else if let Some(sub) = subsystem {
                (
                    vec![
                        "info".to_string(),
                        "--subsystem-match".to_string(),
                        sub.clone(),
                    ],
                    format!("List {} devices", sub),
                )
            } else {
                (
                    vec!["info".to_string(), "--export-db".to_string()],
                    "List all devices".to_string(),
                )
            };
            Ok(Translation {
                command: "udevadm".to_string(),
                args,
                description: desc.clone(),
                permission: PermissionLevel::ReadOnly,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::MountControl {
            action,
            mountpoint,
            filesystem,
        } => {
            let (command, args, desc, permission) = match action.as_str() {
                "list" => {
                    let mut a = Vec::new();
                    if let Some(fs) = filesystem {
                        a.push("-t".to_string());
                        a.push(fs.clone());
                    }
                    let d = if filesystem.is_some() {
                        format!("List {} mounts", filesystem.as_deref().unwrap_or("all"))
                    } else {
                        "List all mounted filesystems".to_string()
                    };
                    ("findmnt".to_string(), a, d, PermissionLevel::Safe)
                }
                "unmount" => {
                    let mp = mountpoint.as_deref().unwrap_or("/mnt");
                    (
                        "fusermount".to_string(),
                        vec!["-u".to_string(), mp.to_string()],
                        format!("Unmount {}", mp),
                        PermissionLevel::Admin,
                    )
                }
                "mount" => {
                    let fs = filesystem.as_deref().unwrap_or("");
                    let mp = mountpoint.as_deref().unwrap_or("/mnt");
                    (
                        "mount".to_string(),
                        vec![fs.to_string(), mp.to_string()],
                        format!("Mount {} on {}", fs, mp),
                        PermissionLevel::Admin,
                    )
                }
                other => {
                    return Err(anyhow!("Unknown mount action: {}", other));
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

        Intent::BootConfig {
            action,
            entry,
            value,
        } => {
            let (args, desc, permission) = match action.as_str() {
                "list" => (
                    vec!["list".to_string()],
                    "List boot entries".to_string(),
                    PermissionLevel::ReadOnly,
                ),
                "default" => {
                    let e = entry.as_deref().unwrap_or("unknown");
                    (
                        vec!["set-default".to_string(), e.to_string()],
                        format!("Set default boot entry to {}", e),
                        PermissionLevel::Admin,
                    )
                }
                "timeout" => {
                    let v = value.as_deref().unwrap_or("5");
                    (
                        vec!["set-timeout".to_string(), v.to_string()],
                        format!("Set boot timeout to {}", v),
                        PermissionLevel::Admin,
                    )
                }
                other => {
                    return Err(anyhow!("Unknown boot config action: {}", other));
                }
            };
            Ok(Translation {
                command: "bootctl".to_string(),
                args,
                description: desc.clone(),
                permission,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::SystemUpdate { action } => {
            let (args, desc, permission) = match action.as_str() {
                "check" => (
                    vec!["check".to_string()],
                    "Check for available system updates".to_string(),
                    PermissionLevel::Safe,
                ),
                "apply" => (
                    vec!["apply".to_string()],
                    "Apply system updates".to_string(),
                    PermissionLevel::Admin,
                ),
                "rollback" => (
                    vec!["rollback".to_string()],
                    "Rollback last system update".to_string(),
                    PermissionLevel::Admin,
                ),
                "status" => (
                    vec!["status".to_string()],
                    "Show current system version and update status".to_string(),
                    PermissionLevel::Safe,
                ),
                other => {
                    return Err(anyhow!("Unknown update action: {}", other));
                }
            };
            Ok(Translation {
                command: "agnos-update".to_string(),
                args,
                description: desc.clone(),
                permission,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::Chmod { path, mode } => {
            let desc = format!("Change permissions of {} to {}", path, mode);
            let permission =
                if path.starts_with("/etc") || path.starts_with("/usr") || path.starts_with("/sys")
                {
                    PermissionLevel::SystemWrite
                } else {
                    PermissionLevel::UserWrite
                };
            Ok(Translation {
                command: "chmod".to_string(),
                args: vec![mode.clone(), path.clone()],
                description: desc.clone(),
                permission,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::Chown { path, owner } => {
            let desc = format!("Change ownership of {} to {}", path, owner);
            Ok(Translation {
                command: "chown".to_string(),
                args: vec![owner.clone(), path.clone()],
                description: desc.clone(),
                permission: PermissionLevel::Admin,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::Symlink { target, link } => {
            let desc = format!("Create symbolic link {} -> {}", link, target);
            Ok(Translation {
                command: "ln".to_string(),
                args: vec!["-s".to_string(), target.clone(), link.clone()],
                description: desc.clone(),
                permission: PermissionLevel::UserWrite,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::Archive {
            action,
            archive,
            files,
        } => {
            let (command, args, desc) = match action.as_str() {
                "tar" | "archive" | "compress" => {
                    let mut a = vec!["-czf".to_string(), archive.clone()];
                    a.extend(files.iter().cloned());
                    let d = format!("Create tar archive {}", archive);
                    ("tar".to_string(), a, d)
                }
                "zip" => {
                    let mut a = vec!["-r".to_string(), archive.clone()];
                    a.extend(files.iter().cloned());
                    let d = format!("Create zip archive {}", archive);
                    ("zip".to_string(), a, d)
                }
                "extract" | "untar" | "decompress" => {
                    let d = format!("Extract archive {}", archive);
                    if archive.ends_with(".zip") {
                        ("unzip".to_string(), vec![archive.clone()], d)
                    } else {
                        (
                            "tar".to_string(),
                            vec!["-xf".to_string(), archive.clone()],
                            d,
                        )
                    }
                }
                "unzip" => {
                    let d = format!("Extract zip archive {}", archive);
                    ("unzip".to_string(), vec![archive.clone()], d)
                }
                _ => {
                    let d = format!("Archive operation on {}", archive);
                    (
                        "tar".to_string(),
                        vec!["-czf".to_string(), archive.clone()],
                        d,
                    )
                }
            };
            Ok(Translation {
                command,
                args,
                description: desc.clone(),
                permission: PermissionLevel::UserWrite,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::Cron {
            action,
            schedule,
            command,
        } => {
            let (args, desc, permission) = match action.as_str() {
                "list" | "show" => (
                    vec!["-l".to_string()],
                    "List crontab entries".to_string(),
                    PermissionLevel::UserWrite,
                ),
                "add" => {
                    let sched = schedule.as_deref().unwrap_or("* * * * *");
                    let cmd = command.as_deref().unwrap_or("echo hello");
                    (
                        vec!["-l".to_string()],
                        format!("Add cron job: {} {}", sched, cmd),
                        PermissionLevel::SystemWrite,
                    )
                }
                "remove" => (
                    vec!["-r".to_string()],
                    "Remove crontab entries".to_string(),
                    PermissionLevel::SystemWrite,
                ),
                "edit" => (
                    vec!["-e".to_string()],
                    "Edit crontab".to_string(),
                    PermissionLevel::SystemWrite,
                ),
                other => {
                    return Err(anyhow!("Unknown cron action: {}", other));
                }
            };
            Ok(Translation {
                command: "crontab".to_string(),
                args,
                description: desc.clone(),
                permission,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::ServiceEnable { service, enable } => {
            let action_str = if *enable { "enable" } else { "disable" };
            let desc = format!("{} service {}", action_str, service);
            Ok(Translation {
                command: "systemctl".to_string(),
                args: vec![action_str.to_string(), service.clone()],
                description: desc.clone(),
                permission: PermissionLevel::Admin,
                explanation: desc,
                mcp: None,
            })
        }

        Intent::EnvVar {
            action,
            name,
            value,
        } => {
            let var_name = name.as_deref().unwrap_or("VAR");
            match action.as_str() {
                "show" | "print" | "echo" => {
                    let desc = format!("Show environment variable {}", var_name);
                    Ok(Translation {
                        command: "env".to_string(),
                        args: vec![],
                        description: desc.clone(),
                        permission: PermissionLevel::Safe,
                        explanation: desc,
                        mcp: None,
                    })
                }
                "set" | "export" => {
                    let val = value.as_deref().unwrap_or("");
                    let desc = format!("Set environment variable {}={}", var_name, val);
                    Ok(Translation {
                        command: "export".to_string(),
                        args: vec![format!("{}={}", var_name, val)],
                        description: desc.clone(),
                        permission: PermissionLevel::UserWrite,
                        explanation: desc,
                        mcp: None,
                    })
                }
                "unset" => {
                    let desc = format!("Unset environment variable {}", var_name);
                    Ok(Translation {
                        command: "unset".to_string(),
                        args: vec![var_name.to_string()],
                        description: desc.clone(),
                        permission: PermissionLevel::UserWrite,
                        explanation: desc,
                        mcp: None,
                    })
                }
                other => Err(anyhow!("Unknown env var action: {}", other)),
            }
        }

        _ => unreachable!("translate_system called with non-system intent"),
    }
}
