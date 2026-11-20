// A simple LSP wrapper to use the opengrep LSP in Zed

use std::env::consts::OS;
use zed_extension_api::{
    self as zed, Command, LanguageServerId, Result, Worktree, settings::LspSettings,
};

// Specify the LSP name as it should be written in the Zed settings json
const LS_NAME: &'static str = "opengrep";

struct OpenGrep;

impl OpenGrep {
    fn language_server_binary_path() -> String {
        match OS {
            "linux" => String::from("~/.local/bin/opengrep"),
            "windows" => String::from(""),
            "macos" => String::from(""),
            _ => String::from(""),
        }
    }
}

impl zed::Extension for OpenGrep {
    fn new() -> Self {
        Self {}
    }
    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let cmd_settings = LspSettings::for_worktree(LS_NAME, worktree)
            .ok()
            .and_then(|f| f.binary);

        let (path, args, env) = match cmd_settings {
            Some(x) => (x.path, x.arguments, x.env),
            None => (None, None, None),
        };

        let command = match path {
            Some(path) => path,
            None => OpenGrep::language_server_binary_path(),
        };

        Ok(zed::Command {
            command,
            args: args.unwrap_or_default(),
            env: env.unwrap_or_default().into_iter().collect(),
        })
    }
}

zed::register_extension!(OpenGrep);
