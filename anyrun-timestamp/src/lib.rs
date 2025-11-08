use abi_stable::std_types::{RString, RVec};
use anyrun_plugin::{HandleResult, Match, PluginInfo, get_matches, handler, info, init};
use chrono::Local;
use matcher::{Matcher, NoopMatcher, SimpleMatch, matcher_static::StaticMatcher};

const PREFIX: &str = ":ts";

#[init]
fn init(_config_dir: RString) {}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Timestamp".into(),
        icon: "appointment-soon".into(),
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
    let now = Local::now();
    let date = now.format("%Y_%m_%d").to_string();
    let datetime = now.format("%Y_%m_%d_%H_%M_%S").to_string();
    let unix = now.timestamp().to_string();

    let matcher = StaticMatcher::new()
        .with_child(
            SimpleMatch::new("YYYY_MM_DD", "x-office-calendar", &format!("copy|{}", date)),
            Box::new(NoopMatcher),
        )
        .with_child(
            SimpleMatch::new(
                "YYYY_MM_DD_HH_MM_SS",
                "x-office-calendar",
                &format!("copy|{}", datetime),
            ),
            Box::new(NoopMatcher),
        )
        .with_child(
            SimpleMatch::new(
                "UNIX_TIMESTAMP",
                "x-office-calendar",
                &format!("copy|{}", unix),
            ),
            Box::new(NoopMatcher),
        );

    Box::new(matcher)
}
