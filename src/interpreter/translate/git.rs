use anyhow::Result;

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_git(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::GitCommit { message, all } => {
            let mut args = vec!["commit".to_string()];
            if *all {
                args.push("-a".to_string());
            }
            args.push("-m".to_string());
            args.push(message.clone());
            Ok(Translation::cmd(
                "git",
                args,
                format!("Commit: {message}"),
                PermissionLevel::UserWrite,
                "Creates a git commit with the specified message",
            ))
        }

        Intent::GitDiff { path, staged } => {
            let mut args = vec!["diff".to_string()];
            if *staged {
                args.push("--staged".to_string());
            }
            if let Some(p) = path {
                args.push("--".to_string());
                args.push(p.clone());
            }
            let desc = if *staged {
                "Show staged changes"
            } else {
                "Show working tree changes"
            };
            Ok(Translation::cmd(
                "git",
                args,
                desc,
                PermissionLevel::ReadOnly,
                "Displays differences between working tree, index, or commits",
            ))
        }

        Intent::GitBranch { name, delete } => {
            let mut args = vec!["branch".to_string()];
            if *delete {
                args.push("-d".to_string());
            }
            if let Some(n) = name {
                args.push(n.clone());
            }
            let desc = match (name, delete) {
                (Some(n), true) => format!("Delete branch {n}"),
                (Some(n), false) => format!("Create branch {n}"),
                (None, _) => "List branches".to_string(),
            };
            Ok(Translation::cmd(
                "git",
                args,
                desc,
                PermissionLevel::UserWrite,
                "Manages git branches: list, create, or delete",
            ))
        }

        Intent::GitStatus => Ok(Translation::cmd(
            "git",
            vec!["status".to_string()],
            "Show repository status",
            PermissionLevel::ReadOnly,
            "Displays the state of the working tree and staging area",
        )),

        Intent::GitLog { count } => {
            let mut args = vec!["log".to_string(), "--oneline".to_string()];
            if let Some(n) = count {
                args.push(format!("-{n}"));
            }
            Ok(Translation::cmd(
                "git",
                args,
                "Show commit history",
                PermissionLevel::ReadOnly,
                "Displays the commit log in abbreviated format",
            ))
        }

        Intent::GitPush { remote, branch } => {
            let mut args = vec!["push".to_string()];
            if let Some(r) = remote {
                args.push(r.clone());
            }
            if let Some(b) = branch {
                args.push(b.clone());
            }
            Ok(Translation::cmd(
                "git",
                args,
                "Push commits to remote",
                PermissionLevel::UserWrite,
                "Uploads local branch commits to the remote repository",
            ))
        }

        Intent::GitPull { remote, branch } => {
            let mut args = vec!["pull".to_string()];
            if let Some(r) = remote {
                args.push(r.clone());
            }
            if let Some(b) = branch {
                args.push(b.clone());
            }
            Ok(Translation::cmd(
                "git",
                args,
                "Pull from remote",
                PermissionLevel::UserWrite,
                "Fetches and integrates changes from the remote repository",
            ))
        }

        Intent::GitCheckout { target } => Ok(Translation::cmd(
            "git",
            vec!["checkout".to_string(), target.clone()],
            format!("Checkout {target}"),
            PermissionLevel::UserWrite,
            "Switches branches or restores working tree files",
        )),

        Intent::GitMerge { branch } => Ok(Translation::cmd(
            "git",
            vec!["merge".to_string(), branch.clone()],
            format!("Merge branch {branch}"),
            PermissionLevel::UserWrite,
            "Joins the specified branch into the current branch",
        )),

        Intent::GitStash { action } => Ok(Translation::cmd(
            "git",
            vec!["stash".to_string(), action.clone()],
            format!("Stash {action}"),
            PermissionLevel::UserWrite,
            "Stashes or restores uncommitted changes",
        )),

        _ => unreachable!(),
    }
}
