use hemtt_common::project::ProjectConfig;
use hemtt_common::reporting::{Code, Processed};

use crate::Number;

use super::Analyze;

impl Analyze for Number {
    fn valid(&self, _: Option<&ProjectConfig>) -> bool {
        true
    }

    fn warnings(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Box<dyn Code>> {
        vec![]
    }

    fn errors(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Box<dyn Code>> {
        vec![]
    }
}
