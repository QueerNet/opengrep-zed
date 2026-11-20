// A simple LSP wrapper to use the opengrep LSP in Zed
use std::env::consts::OS;
use zed_extension_api::{
    self as zed, Command, LanguageServerId, Result, Worktree, serde_json::*, settings::LspSettings,
};

const LS_NAME: &str = "opengrep";

struct OpenGrep;

impl OpenGrep {
    fn default_binary_path() -> String {
        match OS {
            "linux" => "/usr/local/bin/opengrep".to_string(),
            _ => "opengrep".to_string(),
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
            None => OpenGrep::default_binary_path(),
        };

        Ok(zed::Command {
            command,
            args: args.unwrap_or_default(),
            env: env.unwrap_or_default().into_iter().collect(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<Option<Value>> {
        Ok(Some(json!({
            "settings": {
                "pro_intrafile": false,
                "onlyGitDirty": false,
                "timeoutThreshold": 5,
                "ci": false,
                "do_hover": true
            }
        })))
    }
}

zed::register_extension!(OpenGrep);
