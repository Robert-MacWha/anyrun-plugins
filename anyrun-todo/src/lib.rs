mod todo;
use std::fs::{OpenOptions, read_to_string};

use abi_stable::std_types::{RString, RVec};
use anyrun_plugin::{HandleResult, Match, PluginInfo, get_matches, handler, info, init};
use matcher::{
    Matcher, NoopMatcher, SimpleMatch, handler_wildcard_command::WildcardHandler,
    matcher_static::StaticMatcher,
};

const PREFIX: &str = ":todo";
const TODO_FILE: &str = "/home/rmacwha/Documents/todos.txt";

#[init]
fn init(_config_dir: RString) {}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Todo".into(),
        icon: "view-list".into(),
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
    let matches = matcher.get_matches(input);
    let matches: Vec<Match> = matches.into_iter().map(|m| m.into()).collect();
    RVec::from(matches)
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    let matcher = get_matcher();
    matcher.handle(selection.into())
}

fn ensure_todo_file_exists() {
    if let Some(parent) = std::path::Path::new(TODO_FILE).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = OpenOptions::new().create(true).append(true).open(TODO_FILE);
}

fn get_matcher() -> Box<dyn Matcher> {
    ensure_todo_file_exists();

    let todos = read_to_string(TODO_FILE).unwrap_or_default();
    let todos = todos
        .lines()
        .filter_map(|line| todo::Todo::from_str(line))
        .collect::<Vec<todo::Todo>>();

    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut list_matches = StaticMatcher::new();
    for todo in todos.iter().rev().filter(|t| t.completed_at.is_none()) {
        let incomplete_line = todo.to_string();
        let mut completed_todo = todo.clone();
        completed_todo.completed_at = Some(chrono::Local::now().date_naive());
        let completed_line = completed_todo.to_string();

        // Escape special sed characters
        let escaped_incomplete = incomplete_line
            .replace("|", "\\|")
            .replace("[", "\\[")
            .replace("]", "\\]");
        let escaped_complete = completed_line
            .replace("|", "\\|")
            .replace("[", "\\[")
            .replace("]", "\\]");

        list_matches = list_matches.with_child(
            SimpleMatch::new(
                &todo.title,
                "text-x-generic",
                &format!(
                    "exec|sed -i 's|{}|{}|' {}",
                    escaped_incomplete, escaped_complete, TODO_FILE
                ),
            ),
            Box::new(NoopMatcher),
        );
    }
    let matcher = StaticMatcher::new()
        .with_child(
            SimpleMatch::new("list", "text-x-generic", ""),
            Box::new(list_matches),
        )
        .with_child(
            SimpleMatch::new("add", "list-add", ""),
            Box::new(WildcardHandler::new(
                "add {}",
                "",
                &format!(
                    "exec|echo '- [ ] {{}} (created at: {}, completed at: None)' >> {}",
                    date, TODO_FILE
                ),
            )),
        );

    return Box::new(matcher);
}
