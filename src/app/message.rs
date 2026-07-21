use std::{collections::HashSet, pin::Pin};

use chrono::NaiveDate;

use crate::repr::{
    day::Day,
    diff::Diff,
    resource::Resource,
    rule::{Rule, RuleId, UpcomingRuleIndex},
};

#[derive(Debug)]
pub struct RequestWrapper {
    pub response_to: Option<Request>,
    pub content: Request,
}

impl RequestWrapper {
    pub fn new(content: Request) -> Self {
        Self {
            response_to: None,
            content,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Request {
    /// asserts response_to is Some
    RetryWithDeps(Vec<RequestType>),
    Do(Vec<RequestType>),
    Empty,

    ShowDay(NaiveDate),
    LoadDay(NaiveDate),
    CacheDay(NaiveDate, Resource<Day>),
    LoadDayRaw(NaiveDate),

    GenerateNewDayRaw(NaiveDate),
    CacheDayRaw(NaiveDate, Resource<Day>),
    LoadDiff(NaiveDate),
    CacheDiff(NaiveDate, Resource<Diff>),

    LoadRuleIndex,
    CacheRuleIndex(Resource<UpcomingRuleIndex>),
    LoadAllRulesUntilInclude(NaiveDate),
    LoadRule(RuleId),
    CacheRule(RuleId, Resource<Rule>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum RequestType {
    /// only select task that has not yet started
    Fresh(Request),
    /// selects running or parked task
    Any(Request),
}

pub enum Response {
    _Value(Request),
    _Future(Pin<Box<dyn Future<Output = Request> + Send>>),
}

impl Response {
    pub fn value(req: Request) -> Self {
        Self::_Value(req)
    }

    pub fn future<F: Future<Output = Request> + Send + 'static>(fut: F) -> Self {
        Self::_Future(Box::pin(fut))
    }

    pub fn empty() -> Self {
        Self::_Value(Request::Empty)
    }

    pub fn retries(reqs: Vec<RequestType>) -> Self {
        Self::_Value(Request::RetryWithDeps(reqs))
    }

    pub fn retry_fresh(req: Request) -> Self {
        Self::retries(vec![RequestType::Fresh(req)])
    }

    pub fn retry_any(req: Request) -> Self {
        Self::retries(vec![RequestType::Any(req)])
    }
}
