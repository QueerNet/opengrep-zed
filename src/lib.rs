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
            "linux" => "/home/mj/.local/bin/opengrep".to_string(),
            _ => "opengrep".to_string(),
        }
    }

    fn initialization_options(user: Option<Value>, settings: Value) -> Value {
        let mut options = match user {
            Some(Value::Object(map)) => Value::Object(map),
            _ => Value::Object(Default::default()),
        };
        if let Value::Object(map) = &mut options {
            map.entry("settings").or_insert(settings);
        }
        options
    }

    fn workspace_settings(worktree: &zed::Worktree) -> Value {
        let mut settings = LspSettings::for_worktree(LS_NAME, worktree)
            .ok()
            .and_then(|settings| settings.settings)
            .unwrap_or_else(|| Value::Object(Default::default()));
        OpenGrep::resolve_relative_paths(&mut settings, &worktree.root_path());
        settings
    }

    fn resolve_relative_paths(settings: &mut Value, root: &str) {
        let Some(xml) = settings.get_mut("xml") else {
            return;
        };
        if let Some(catalogs) = xml.get_mut("catalogs").and_then(Value::as_array_mut) {
            for catalog in catalogs.iter_mut() {
                if let Some(path) = catalog.as_str() {
                    *catalog = Value::String(OpenGrep::absolute(path, root));
                }
            }
        }
        if let Some(associations) = xml
            .get_mut("fileAssociations")
            .and_then(Value::as_array_mut)
        {
            for association in associations.iter_mut() {
                if let Some(system_id) = association.get_mut("systemId") {
                    if let Some(path) = system_id.as_str() {
                        *system_id = Value::String(OpenGrep::absolute(path, root));
                    }
                }
            }
        }
    }

    fn is_absolute(path: &str) -> bool {
        let bytes = path.as_bytes();
        path.starts_with('/')
            || path.starts_with('\\')
            || OpenGrep::has_uri_scheme(path)
            || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
    }

    fn absolute(path: &str, root: &str) -> String {
        if OpenGrep::is_absolute(path) {
            return path.to_string();
        }
        let root = root.trim_end_matches(['/', '\\']);
        let relative = path.trim_start_matches("./");
        format!("{root}/{relative}")
    }

    fn has_uri_scheme(path: &str) -> bool {
        let Some((scheme, _)) = path.split_once(':') else {
            return false;
        };
        scheme.len() > 1
            && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
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
        worktree: &zed::Worktree,
    ) -> Result<Option<Value>> {
        let user = LspSettings::for_worktree(LS_NAME, worktree)
            .ok()
            .and_then(|settings| settings.initialization_options);
        Ok(Some(OpenGrep::initialization_options(
            user,
            OpenGrep::workspace_settings(worktree),
        )))
    }
}

zed::register_extension!(OpenGrep);
