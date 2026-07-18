use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, hash_map},
    error::Error,
    sync::Arc,
    time::Duration,
};

use chrono::{Datelike, NaiveDate, Weekday};
use log::warn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app::message::{Request, RequestType, Response},
    macros::simple_id,
    repr::{common::Colour, entry::Entry, resource::Resource, sticker::Sticker, storage::Storage},
};

impl RuleManager {
    pub fn load_all_rules_until_include(&self, date: NaiveDate) -> Response {
        let mut deps = HashSet::new();

        match &self.index {
            Some(Resource::Failed(_)) => todo!(),
            Some(Resource::Loaded(index)) => index
                .0
                .iter()
                .take_while(|entry| entry.date <= date)
                .for_each(|entry| {
                    deps.insert(RequestType::Any(Request::LoadRule(entry.id)));
                }),
            Some(Resource::Loading) | None => {
                deps.insert(RequestType::Any(Request::LoadRuleIndex));
            }
        }

        if deps.is_empty() {
            Response::empty()
        } else {
            Response::value(Request::RetryWithDeps(deps.into_iter().collect()))
        }
    }

    pub fn load_rule(&mut self, id: RuleId, storage: Arc<dyn Storage>) -> Response {
        match self.cache.entry(id) {
            hash_map::Entry::Occupied(v) if v.get().is_loading() => {}
            hash_map::Entry::Occupied(_) => return Response::empty(), // short circuiting
            hash_map::Entry::Vacant(v) => {
                v.insert(Resource::Loading);
            }
        }

        Response::future(async move {
            match Self::load(id, storage).await {
                Ok(rule) => Request::RetryWithDeps(vec![RequestType::Fresh(Request::CacheRule(
                    id,
                    Resource::Loaded(rule),
                ))]),
                Err(err) => Request::RetryWithDeps(vec![RequestType::Fresh(Request::CacheRule(
                    id,
                    Resource::Failed(err.into()),
                ))]),
            }
        })
    }

    pub fn cache_rule(&mut self, id: RuleId, rule: Resource<Rule>) -> Response {
        self.cache.insert(id, rule);
        Response::value(Request::Empty)
    }

    pub fn load_rule_index(&mut self, storage: Arc<dyn Storage>) -> Response {
        match &self.index {
            Some(v) if v.is_loading() => {}
            Some(_) => return Response::empty(), // short circuiting
            None => {
                self.index = Some(Resource::Loading);
            }
        }

        Response::future(async move {
            match Self::load_index(storage).await {
                Ok(index) => Request::RetryWithDeps(vec![RequestType::Fresh(
                    Request::CacheRuleIndex(Resource::Loaded(index)),
                )]),
                Err(err) => Request::RetryWithDeps(vec![RequestType::Fresh(
                    Request::CacheRuleIndex(Resource::Failed(err.into())),
                )]),
            }
        })
    }

    pub fn cache_rule_index(&mut self, index: Resource<UpcomingRuleIndex>) -> Response {
        self.index = Some(index);
        Response::empty()
    }
}

pub struct RuleManager {
    cache: HashMap<RuleId, Resource<Rule>>,
    index: Option<Resource<UpcomingRuleIndex>>,
}

impl Default for RuleManager {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            index: None,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub struct RuleId(Uuid);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Rule {
    id: RuleId,
    label: String,
    duration: Duration,
    colour: Colour,
    recurrence: Recurrence,
    first_occurence: NaiveDate,
    /// Date of last occurence (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    cut_off: Option<NaiveDate>,
    default_sticker: Sticker,
}

simple_id!(Rule);

impl Rule {
    pub fn id(&self) -> RuleId {
        self.id
    }

    pub fn default_sticker(&self) -> &Sticker {
        &self.default_sticker
    }

    pub fn is_on_day(&self, date: NaiveDate) -> bool {
        self.first_occurence <= date
            && self.cut_off.is_none_or(|cut_off| date <= cut_off)
            && match self.recurrence {
                Recurrence::Once => self.first_occurence == date,
                Recurrence::Weekly {
                    mon,
                    tue,
                    wed,
                    thu,
                    fri,
                    sat,
                    sun,
                } => match date.weekday() {
                    Weekday::Mon => mon,
                    Weekday::Tue => tue,
                    Weekday::Wed => wed,
                    Weekday::Thu => thu,
                    Weekday::Fri => fri,
                    Weekday::Sat => sat,
                    Weekday::Sun => sun,
                },
                Recurrence::Interval { value } => {
                    (date - self.first_occurence).num_days() % value as i64 == 0
                }
            }
    }
}

impl From<Rule> for Entry {
    fn from(value: Rule) -> Self {
        Entry {
            id: value.id,
            sticker: value.default_sticker.clone(),
            sessions: Vec::new(),
            ongoing_session: None,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Recurrence {
    #[serde(rename = "once")]
    Once,
    #[serde(rename = "weekly")]
    Weekly {
        mon: bool,
        tue: bool,
        wed: bool,
        thu: bool,
        fri: bool,
        sat: bool,
        sun: bool,
    },
    #[serde(rename = "interval")]
    Interval {
        /// days
        value: u64,
    },
}

/// List of all rules that are
/// - Active: is scheduled to be at a later date (excluding diff entries)
/// - In order of first date used
#[derive(Default, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct UpcomingRuleIndex(pub BTreeSet<RuleIndexEntry>);

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuleIndexEntry {
    date: NaiveDate,
    id: RuleId,
}

impl RuleManager {
    pub async fn load_index(
        storage: Arc<dyn Storage>,
    ) -> Result<UpcomingRuleIndex, Box<dyn Error + Send + Sync>> {
        match storage.read("rules", "index.json").await? {
            Some(bytes) => Ok(serde_json::from_slice(&bytes)?),
            None => Ok(UpcomingRuleIndex::default()),
        }
    }

    pub async fn write_index(
        &self,
        storage: Arc<dyn Storage>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self.index {
            Some(Resource::Loaded(index)) => {
                let bytes = serde_json::to_vec(index)?;
                storage.write("rules", "index.json", &bytes).await?;
            }
            Some(Resource::Loading) => warn!("Writing rule index but is loading"),
            Some(Resource::Failed(err)) => warn!("Writing rule index but is failed : {err}"),
            None => warn!("Writing rule index but is not in cache"),
        };

        Ok(())
    }
}

impl RuleManager {
    pub fn get_loaded_matches(&self, date: NaiveDate) -> Vec<&Rule> {
        match &self.index {
            None => vec![],
            Some(Resource::Loading) => vec![],
            Some(Resource::Failed(_)) => todo!(),
            Some(Resource::Loaded(index)) => index
                .0
                .iter()
                .take_while(|entry| entry.date <= date)
                .filter_map(|entry| self.get(&entry.id))
                .filter_map(|rule| match rule {
                    Resource::Loading => None,
                    Resource::Failed(_) => todo!(),
                    Resource::Loaded(rule) => Some(rule),
                })
                .filter(|rule| rule.is_on_day(date))
                .collect(),
        }
    }

    pub fn get(&self, id: &RuleId) -> Option<&Resource<Rule>> {
        self.cache.get(id)
    }

    pub async fn load(
        id: RuleId,
        storage: Arc<dyn Storage>,
    ) -> Result<Rule, Box<dyn Error + Send + Sync>> {
        match storage
            .read("rules", &format!("{}.rule.json", id.0))
            .await?
        {
            Some(bytes) => {
                Ok::<Rule, Box<dyn Error + Send + Sync>>(serde_json::from_slice(&bytes)?)
            }
            None => todo!(),
        }
    }

    pub async fn write(
        &self,
        id: &RuleId,
        storage: Arc<dyn Storage>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.cache.get(id) {
            Some(Resource::Loaded(diff)) => {
                let bytes = serde_json::to_vec(diff)?;
                storage
                    .write("rules", &format!("{}.rule.json", id.0), &bytes)
                    .await?;
            }
            Some(Resource::Loading) => warn!("Writing rule for {id:?} but is loading"),
            Some(Resource::Failed(err)) => warn!("Writing rule for {id:?} but is failed : {err}"),
            None => warn!("Writing rule for {id:?} but is not in cache"),
        };

        Ok(())
    }
}

impl RuleManager {
    pub fn is_loaded_until(&self, date: NaiveDate) -> bool {
        let index = match &self.index {
            Some(Resource::Loading) | None => return false,
            Some(Resource::Failed(_)) => todo!(),
            Some(Resource::Loaded(index)) => index,
        };

        index
            .0
            .iter()
            .take_while(|entry| entry.date <= date)
            .all(|entry| {
                self.get(&entry.id).is_some_and(|value| match value {
                    Resource::Loading => false,
                    Resource::Failed(_) => todo!(),
                    Resource::Loaded(_) => true,
                })
            })
    }
}
