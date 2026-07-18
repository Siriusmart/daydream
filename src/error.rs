use std::fmt::Display;

use crate::repr::rule::RuleId;

#[derive(Debug)]
pub enum Error {
    RuleNotFound(RuleId),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl std::error::Error for Error {}
