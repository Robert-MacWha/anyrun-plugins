pub mod handler_display;
pub mod handler_display_command;
pub mod handler_wildcard_command;
pub mod matcher_static;
use abi_stable::std_types::ROption;
use anyrun_plugin::{HandleResult, Match};

pub trait Matcher {
    fn get_matches(&self, input: Vec<&str>) -> Vec<SimpleMatch>;
    fn handle(&self, selection: SimpleMatch) -> HandleResult;
}

pub struct NoopMatcher;

impl Matcher for NoopMatcher {
    fn get_matches(&self, _input: Vec<&str>) -> Vec<SimpleMatch> {
        vec![]
    }

    fn handle(&self, _selection: SimpleMatch) -> HandleResult {
        HandleResult::Refresh(false)
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct SimpleMatch {
    pub title: String,
    pub icon: String,
    pub description: String,
}

impl SimpleMatch {
    pub fn new(title: &str, icon: &str, description: &str) -> Self {
        SimpleMatch {
            title: title.to_string(),
            icon: icon.to_string(),
            description: description.to_string(),
        }
    }
}

impl From<SimpleMatch> for Match {
    fn from(simple_match: SimpleMatch) -> Self {
        let icon = if simple_match.icon.is_empty() {
            ROption::RNone
        } else {
            ROption::RSome(simple_match.icon.into())
        };
        let description = if simple_match.description.is_empty() {
            ROption::RNone
        } else {
            ROption::RSome(simple_match.description.into())
        };
        Match {
            title: simple_match.title.into(),
            icon,
            description,
            use_pango: false,
            id: ROption::RNone,
        }
    }
}

impl From<Match> for SimpleMatch {
    fn from(m: Match) -> Self {
        SimpleMatch {
            title: m.title.to_string(),
            icon: m.icon.unwrap_or_default().to_string(),
            description: m.description.unwrap_or_default().to_string(),
        }
    }
}
