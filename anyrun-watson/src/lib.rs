mod watson;
mod watson_matcher;
use std::collections::HashSet;

use abi_stable::std_types::{RString, RVec};
use anyrun_plugin::{HandleResult, Match, PluginInfo, get_matches, handler, info, init};
use matcher::{
    Matcher, NoopMatcher, SimpleMatch, handler_display_command::CommandDisplayHandler,
    matcher_static::StaticMatcher,
};

use crate::{watson::get_current_status, watson_matcher::ProjectMatcher};

const PREFIX: &str = ":tt";

#[init]
fn init(_config_dir: RString) {}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Watson".into(),
        icon: "alarm".into(),
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

fn get_matcher() -> Box<dyn Matcher> {
    let mut matcher = StaticMatcher::new();

    let current_status = get_current_status();
    match current_status {
        Some((project, _tags, _)) => {
            matcher = matcher.with_child(
                SimpleMatch::new(
                    &format!("stop {}", project),
                    "media-playback-stop",
                    "exec|watson stop",
                ),
                Box::new(NoopMatcher),
            );
        }
        None => {}
    };
    matcher = matcher.with_child(
        SimpleMatch::new("start", "media-playback-start", ""),
        get_start_matcher(),
    );

    matcher = matcher.with_child(
        SimpleMatch::new("log", "format-justify-left", ""),
        Box::new(CommandDisplayHandler::new("watson log -d -c")),
    );

    matcher = matcher.with_child(
        SimpleMatch::new("report", "document-properties", ""),
        get_report_matcher(),
    );

    Box::new(matcher)
}

fn get_start_matcher() -> Box<dyn Matcher> {
    let known_tags = watson::get_tags();
    let combinations = watson::get_project_tag_combinations();
    let mut projects = combinations
        .iter()
        .map(|combo| combo.project.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    projects.sort();

    let matcher = ProjectMatcher::new(projects, known_tags, combinations);
    Box::new(matcher)
}

fn get_report_matcher() -> Box<dyn Matcher> {
    let matcher = StaticMatcher::new()
        .with_child(
            SimpleMatch::new("day", "view-calendar-day", ""),
            Box::new(CommandDisplayHandler::new("watson report -d")),
        )
        .with_child(
            SimpleMatch::new("week", "view-calendar-week", ""),
            Box::new(CommandDisplayHandler::new("watson report -w")),
        )
        .with_child(
            SimpleMatch::new("month", "view-calendar-month", ""),
            Box::new(CommandDisplayHandler::new("watson report -m")),
        );

    Box::new(matcher)
}
