use anyhow::Result;

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_stiva(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::StivaRun { image, name, args } => {
            let mut cmd_args = vec!["run".to_string()];
            if let Some(n) = name {
                cmd_args.push("--name".to_string());
                cmd_args.push(n.clone());
            }
            cmd_args.push(image.clone());
            cmd_args.extend(args.clone());
            Ok(Translation::cmd(
                "stiva",
                cmd_args,
                format!("Run container from image {image}"),
                PermissionLevel::SystemWrite,
                "Runs a new container from the specified image",
            ))
        }

        Intent::StivaStop { id } => Ok(Translation::cmd(
            "stiva",
            vec!["stop".to_string(), id.clone()],
            format!("Stop container {id}"),
            PermissionLevel::SystemWrite,
            "Stops a running container",
        )),

        Intent::StivaPs { all } => {
            let mut args = vec!["ps".to_string()];
            if *all {
                args.push("-a".to_string());
            }
            Ok(Translation::cmd(
                "stiva",
                args,
                "List containers".to_string(),
                PermissionLevel::ReadOnly,
                "Lists running containers",
            ))
        }

        Intent::StivaRm { id, force } => {
            let mut args = vec!["rm".to_string()];
            if *force {
                args.push("-f".to_string());
            }
            args.push(id.clone());
            Ok(Translation::cmd(
                "stiva",
                args,
                format!("Remove container {id}"),
                PermissionLevel::SystemWrite,
                "Removes a container",
            ))
        }

        Intent::StivaPull { image } => Ok(Translation::cmd(
            "stiva",
            vec!["pull".to_string(), image.clone()],
            format!("Pull image {image}"),
            PermissionLevel::UserWrite,
            "Pulls a container image from a registry",
        )),

        Intent::StivaImages => Ok(Translation::cmd(
            "stiva",
            vec!["images".to_string()],
            "List images".to_string(),
            PermissionLevel::ReadOnly,
            "Lists available container images",
        )),

        Intent::StivaRmi { image } => Ok(Translation::cmd(
            "stiva",
            vec!["rmi".to_string(), image.clone()],
            format!("Remove image {image}"),
            PermissionLevel::SystemWrite,
            "Removes a container image",
        )),

        Intent::StivaBuild { path, tag } => {
            let mut args = vec!["build".to_string(), path.clone()];
            if let Some(t) = tag {
                args.push("-t".to_string());
                args.push(t.clone());
            }
            Ok(Translation::cmd(
                "stiva",
                args,
                format!("Build image from {path}"),
                PermissionLevel::UserWrite,
                "Builds a container image from a build context",
            ))
        }

        Intent::StivaLogs { id, tail } => {
            let mut args = vec!["logs".to_string(), id.clone()];
            if let Some(n) = tail {
                args.push("--tail".to_string());
                args.push(n.to_string());
            }
            Ok(Translation::cmd(
                "stiva",
                args,
                format!("Show logs for container {id}"),
                PermissionLevel::ReadOnly,
                "Displays container log output",
            ))
        }

        Intent::StivaExec { id, command, args } => {
            let mut cmd_args = vec!["exec".to_string(), id.clone(), command.clone()];
            cmd_args.extend(args.clone());
            Ok(Translation::cmd(
                "stiva",
                cmd_args,
                format!("Execute {command} in container {id}"),
                PermissionLevel::SystemWrite,
                "Executes a command inside a running container",
            ))
        }

        Intent::StivaInspect { target } => Ok(Translation::cmd(
            "stiva",
            vec!["inspect".to_string(), target.clone()],
            format!("Inspect {target}"),
            PermissionLevel::ReadOnly,
            "Shows detailed information about a container or image",
        )),

        Intent::StivaAnsamblu { action, file } => {
            let mut args = vec!["ansamblu".to_string(), action.clone()];
            if let Some(f) = file {
                args.push("-f".to_string());
                args.push(f.clone());
            }
            Ok(Translation::cmd(
                "stiva",
                args,
                format!("Ansamblu {action}"),
                PermissionLevel::SystemWrite,
                "Manages multi-container applications via ansamblu (compose)",
            ))
        }

        _ => unreachable!("translate_stiva called with non-stiva intent"),
    }
}
