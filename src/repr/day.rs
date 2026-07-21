use std::{
    collections::{HashMap, HashSet, hash_map},
    error::Error,
    sync::Arc,
};

use chrono::{Local, NaiveDate};
use iced::{Rectangle, Renderer, widget::canvas::Frame};
use linked_hash_map::LinkedHashMap;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::{
    app::message::{Request, RequestType, Response},
    macros::simple_id,
    repr::{
        diff::{Diff, DiffManager},
        entry::Entry,
        resource::Resource,
        rule::{Rule, RuleId, RuleManager},
        storage::Storage,
    },
};

#[derive(Default, Clone)]
pub struct DayManager {
    /// the day file, with the guarantee that all diffs, rules and entries files are in cache
    cache: HashMap<NaiveDate, Resource<Day>>,
    /// just the raw day file content
    raw_cache: HashMap<NaiveDate, Resource<Day>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Day {
    id: NaiveDate,
    /// in order access time, later is higher in layer
    entries: LinkedHashMap<RuleId, Entry>,
}

simple_id!(Day);

impl Day {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            id: date,
            entries: LinkedHashMap::new(),
        }
    }

    pub fn draw(
        &self,
        rules: &HashMap<RuleId, Rule>,
        renderer: &Renderer,
        bounds: Rectangle,
    ) -> Vec<Frame> {
        self.entries
            .values()
            .map(|entry| entry.draw(renderer, bounds, rules))
            .collect()
    }

    pub fn extract_rules(&self, rules: &RuleManager) -> HashMap<RuleId, Rule> {
        HashMap::from_iter(
            self.entries
                .values()
                .map(|entry| (entry.id, rules.get(&entry.id).expect("rule to be loaded")))
                .map(|(id, resource)| {
                    if let Resource::Loaded(v) = resource {
                        (id, v.clone())
                    } else {
                        panic!("rule not loaded")
                    }
                }),
        )
    }
}

impl DayManager {
    pub fn load_day(
        &mut self,
        date: NaiveDate,
        diffs: &DiffManager,
        rules: &RuleManager,
    ) -> Response {
        match self.cache.entry(date) {
            // must be from previous pass due to dedup
            hash_map::Entry::Occupied(v) if v.get().is_loading() => {}
            hash_map::Entry::Occupied(_) => return Response::empty(), // short circuiting
            hash_map::Entry::Vacant(v) => {
                v.insert(Resource::Loading);
            }
        }

        let day_raw = self.raw_cache.get(&date);
        let diff = diffs.get(&date);

        let mut deps = HashSet::new();

        match day_raw {
            Some(Resource::Loading) => {
                // wait for loading to finish
                deps.insert(RequestType::Any(Request::LoadDayRaw(date)));
            }
            Some(Resource::Failed(_)) => todo!(),
            Some(Resource::Loaded(day)) => {
                day.entries.keys().for_each(|entry_id| {
                    if rules.get(entry_id).is_none() {
                        deps.insert(RequestType::Any(Request::LoadRule(*entry_id)));
                    }
                });
            }
            None => {
                deps.insert(RequestType::Any(Request::LoadDayRaw(date)));
            }
        }

        match diff {
            Some(Resource::Failed(_)) => todo!(),
            Some(Resource::Loaded(_diff)) => {}
            Some(Resource::Loading) | None => {
                // load or wait for loading to finish
                deps.insert(RequestType::Any(Request::LoadDiff(date)));
            }
        }

        if deps.is_empty() {
            if let Some(Resource::Loaded(day)) = day_raw {
                return Response::retry_fresh(Request::CacheDay(
                    date,
                    Resource::Loaded(day.clone()),
                ));
            } else {
                unreachable!("already asserted day is loaded")
            }
        }

        Response::value(Request::RetryWithDeps(deps.into_iter().collect()))
    }

    pub fn cache_day(&mut self, date: NaiveDate, value: Resource<Day>) -> Response {
        self.cache.insert(date, value);
        Response::empty()
    }

    pub fn load_day_raw(&mut self, date: NaiveDate, storage: Arc<dyn Storage>) -> Response {
        match self.raw_cache.entry(date) {
            // must be from previous pass due to dedup
            hash_map::Entry::Occupied(v) if v.get().is_loading() => {}
            hash_map::Entry::Occupied(_) => {
                return Response::empty(); // short circuit
            }
            hash_map::Entry::Vacant(v) => {
                v.insert(Resource::Loading);
            }
        }

        Response::future(async move {
            Request::RetryWithDeps(vec![RequestType::Fresh(Request::CacheDayRaw(
                date,
                match Self::load(date, storage).await {
                    Ok(Some(day)) => Resource::Loaded(day),
                    Ok(None) if date < Local::now().date_naive() => {
                        // in the past
                        Resource::Loaded(Day::new(date))
                    }
                    Ok(None) => {
                        return Request::RetryWithDeps(vec![RequestType::Any(
                            Request::GenerateNewDayRaw(date),
                        )]);
                    }
                    Err(err) => Resource::Failed(err.into()),
                },
            ))])
        })
    }

    pub fn cache_day_raw(&mut self, date: NaiveDate, value: Resource<Day>) -> Response {
        self.raw_cache.insert(date, value);
        Response::empty()
    }

    pub fn generate_new_day_raw(&mut self, date: NaiveDate, rules: &RuleManager) -> Response {
        match self.raw_cache.get(&date) {
            // must be from previous pass due to dedup
            Some(Resource::Loading) => {}
            Some(Resource::Loaded(_)) => return Response::empty(), // short circuiting
            Some(Resource::Failed(_)) => todo!(),
            None => {}
        }

        if !rules.is_loaded_until(date) {
            return Response::retry_any(Request::LoadAllRulesUntilInclude(date));
        }

        let rules = rules.get_loaded_matches(date);
        let day = Day {
            id: date,
            entries: LinkedHashMap::from_iter(
                rules.into_iter().map(|rule| (rule.id(), Entry::new(rule))),
            ),
        };

        Response::retry_fresh(Request::CacheDayRaw(date, Resource::Loaded(day)))
    }
}

impl DayManager {
    pub fn get(&self, date: &NaiveDate) -> Option<&Resource<Day>> {
        self.cache.get(date)
    }

    pub async fn load(
        date: NaiveDate,
        storage: Arc<dyn Storage>,
    ) -> Result<Option<Day>, Box<dyn Error + Send + Sync>> {
        match storage
            .read("days", &format!("{}.day.json", date.format("%Y-%m-%d")))
            .await?
        {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    pub async fn write(
        &self,
        date: &NaiveDate,
        storage: Arc<dyn Storage>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.cache.get(date) {
            Some(Resource::Loaded(diff)) => {
                let bytes = serde_json::to_vec(diff)?;
                storage
                    .write(
                        "days",
                        &format!("{}.day.json", date.format("%Y-%m-%d")),
                        &bytes,
                    )
                    .await?;
            }
            Some(Resource::Loading) => warn!("Writing day for {date} but is loading"),
            Some(Resource::Failed(err)) => warn!("Writing day for {date} but is failed : {err}"),
            None => warn!("Writing day for {date} but is not in cache"),
        };

        Ok(())
    }
}

impl Day {
    pub fn apply(mut self, diff: &Diff) -> Self {
        let old = std::mem::take(&mut self.entries);

        self.entries = LinkedHashMap::from_iter(
            old.into_iter()
                .filter(|(_, entry)| !diff.removed_entries.contains(&entry.id)),
        );

        self
    }
}
