use anyhow::Result;

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_user(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::UserAdd {
            username,
            shell,
            home,
        } => {
            let mut args = vec![username.clone()];
            if let Some(s) = shell {
                args.insert(0, "-s".to_string());
                args.insert(1, s.clone());
            }
            if let Some(h) = home {
                args.insert(0, "-d".to_string());
                args.insert(1, h.clone());
            }
            args.insert(0, "useradd".to_string());
            Ok(Translation::cmd(
                "sudo",
                args,
                format!("Add user {username}"),
                PermissionLevel::Admin,
                "Creates a new user account on the system",
            ))
        }

        Intent::UserDelete {
            username,
            remove_home,
        } => {
            let mut args = vec!["userdel".to_string()];
            if *remove_home {
                args.push("-r".to_string());
            }
            args.push(username.clone());
            Ok(Translation::cmd(
                "sudo",
                args,
                format!("Delete user {username}"),
                PermissionLevel::Admin,
                "Removes a user account from the system",
            ))
        }

        Intent::UserMod {
            username,
            shell,
            groups,
        } => {
            let mut args = vec!["usermod".to_string()];
            if let Some(s) = shell {
                args.push("-s".to_string());
                args.push(s.clone());
            }
            if let Some(g) = groups {
                args.push("-aG".to_string());
                args.push(g.clone());
            }
            args.push(username.clone());
            Ok(Translation::cmd(
                "sudo",
                args,
                format!("Modify user {username}"),
                PermissionLevel::Admin,
                "Modifies an existing user account's properties",
            ))
        }

        Intent::Passwd { username } => {
            let mut args = vec!["passwd".to_string()];
            if let Some(u) = username {
                args.push(u.clone());
            }
            let desc = username
                .as_ref()
                .map(|u| format!("Change password for {u}"))
                .unwrap_or_else(|| "Change current user password".to_string());
            Ok(Translation::cmd(
                "sudo",
                args,
                desc,
                PermissionLevel::Admin,
                "Changes the password for a user account",
            ))
        }

        Intent::GroupAdd { groupname } => Ok(Translation::cmd(
            "sudo",
            vec!["groupadd".to_string(), groupname.clone()],
            format!("Add group {groupname}"),
            PermissionLevel::Admin,
            "Creates a new group on the system",
        )),

        Intent::GroupDelete { groupname } => Ok(Translation::cmd(
            "sudo",
            vec!["groupdel".to_string(), groupname.clone()],
            format!("Delete group {groupname}"),
            PermissionLevel::Admin,
            "Removes a group from the system",
        )),

        Intent::GroupList { username } => {
            let mut args = Vec::new();
            if let Some(u) = username {
                args.push(u.clone());
            }
            Ok(Translation::cmd(
                "groups",
                args,
                username
                    .as_ref()
                    .map(|u| format!("List groups for {u}"))
                    .unwrap_or_else(|| "List groups for current user".to_string()),
                PermissionLevel::ReadOnly,
                "Displays the groups a user belongs to",
            ))
        }

        _ => unreachable!(),
    }
}
