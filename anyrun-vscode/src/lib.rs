use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use abi_stable::std_types::{RString, RVec};
use anyrun_plugin::{HandleResult, Match, PluginInfo, get_matches, handler, info, init};
use matcher::{Matcher, NoopMatcher, SimpleMatch, matcher_static::StaticMatcher};
use serde::Deserialize;

#[derive(Deserialize)]
struct RecentlyOpened {
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "folderUri")]
    folder_uri: Option<String>,
    #[serde(rename = "workspace")]
    workspace: Option<String>,
}

const PREFIX: &str = ":vs";
const MAX_RESULTS: usize = 10;

#[init]
fn init(_config_dir: RString) {}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "VSCode Workspace".into(),
        icon: "folder".into(),
    }
}

#[get_matches]
fn get_matches(input: RString) -> RVec<Match> {
    let input = input.trim().to_lowercase();

    if !input.starts_with(PREFIX) {
        return RVec::new();
    }

    let input = input
        .trim_start_matches(PREFIX)
        .split_ascii_whitespace()
        .collect::<Vec<_>>();

    let matcher = get_matcher();
    let matcher = match matcher {
        Ok(m) => m,
        Err(_) => {
            return RVec::from(vec![
                SimpleMatch::new("Could not retrieve recent VSCode workspaces", "error", "").into(),
            ]);
        }
    };
    let matches = matcher.get_matches(input);
    let matches: Vec<Match> = matches.into_iter().map(|m| m.into()).collect();
    RVec::from(matches)
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    let matcher = get_matcher();
    let matcher = match matcher {
        Ok(m) => m,
        Err(_) => {
            return HandleResult::Refresh(false);
        }
    };
    matcher.handle(selection.into())
}

fn get_matcher() -> Result<Box<dyn Matcher>, String> {
    let recent_projects = match get_recent_projects() {
        Ok(projects) => projects,
        Err(_) => return Err("Could not retrieve recent VSCode workspaces".into()),
    };

    let mut matcher = StaticMatcher::new().with_max_results(MAX_RESULTS);
    for project in recent_projects {
        let name = project
            .rsplit(std::path::MAIN_SEPARATOR)
            .next()
            .unwrap_or(&project)
            .to_string();
        let command = format!("exec|code \"{}\"", project);
        matcher = matcher.with_child(
            SimpleMatch::new(&name, "folder", &command),
            Box::new(NoopMatcher),
        );
    }

    Ok(Box::new(matcher))
}

fn get_recent_projects() -> Result<Vec<String>, String> {
    let path = vscode_state_path().ok_or("Could not locate VSCode state file path")?;
    if !path.exists() {
        return Err("VSCode state file not found".into());
    }

    // Check if sqlite3 is available
    if Command::new("sqlite3")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        return Err("sqlite3 command not found".into());
    }

    let output = Command::new("sqlite3")
        .arg(&path)
        .arg("SELECT value FROM ItemTable WHERE key='history.recentlyOpenedPathsList';")
        .output()
        .map_err(|_| "Failed to execute sqlite3".to_string())?;

    let text = String::from_utf8(output.stdout)
        .map_err(|_| "Invalid UTF-8 in sqlite output".to_string())?;

    let data: RecentlyOpened = serde_json::from_str(&text)
        .map_err(|_| "Failed to parse VSCode history JSON".to_string())?;

    let mut result = Vec::new();
    for entry in data.entries {
        let Some(uri) = entry.folder_uri.or(entry.workspace) else {
            continue;
        };
        result.push(uri.trim_start_matches("file://").to_string());
    }

    if result.is_empty() {
        return Err("No recent workspaces found".into());
    }

    Ok(result)
}

fn vscode_state_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let mut path = dirs::home_dir()?;
        path.push(".config/Code/User/globalStorage/state.vscdb");
        return Some(path);
    }

    #[cfg(target_os = "macos")]
    {
        let mut path = dirs::home_dir()?;
        path.push("Library/Application Support/Code/User/globalStorage/state.vscdb");
        return Some(path);
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var_os("APPDATA")?;
        let mut path = PathBuf::from(appdata);
        path.push("Code/User/globalStorage/state.vscdb");
        return Some(path);
    }

    #[allow(unreachable_code)]
    None
}
