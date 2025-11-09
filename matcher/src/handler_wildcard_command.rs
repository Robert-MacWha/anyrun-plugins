use anyrun_plugin::HandleResult;

use crate::{Matcher, SimpleMatch};

/// Handler that matches any input as a wildcard and creates a match based on a template.
/// The title and description can include "{}" as a placeholder for the input text.
pub struct WildcardHandler {
    title: String,
    icon: String,
    description: String,
}

impl WildcardHandler {
    pub fn new(title: &str, icon: &str, description: &str) -> Self {
        WildcardHandler {
            title: title.to_string(),
            icon: icon.to_string(),
            description: description.to_string(),
        }
    }
}

impl Matcher for WildcardHandler {
    fn get_matches(&self, input: Vec<&str>) -> Vec<SimpleMatch> {
        let text = input.join(" ");

        let title = if self.title.contains("{}") {
            self.title.replace("{}", &text)
        } else {
            self.title.clone()
        };

        let description = if self.description.contains("{}") {
            self.description.replace("{}", &text)
        } else {
            self.description.clone()
        };

        vec![SimpleMatch::new(&title, &self.icon, &description)]
    }

    fn handle(&self, _selection: SimpleMatch) -> HandleResult {
        HandleResult::Close
    }
}
