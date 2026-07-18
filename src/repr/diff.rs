use std::{
    collections::{HashMap, HashSet, hash_map},
    error::Error,
    sync::Arc,
};

use chrono::NaiveDate;
use linked_hash_set::LinkedHashSet;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::{
    app::message::{Request, RequestType, Response},
    repr::{
        resource::Resource,
        rule::{RuleId, RuleManager},
        storage::Storage,
    },
};

#[derive(Default)]
pub struct DiffManager {
    cache: HashMap<NaiveDate, Resource<Diff>>,
}

impl DiffManager {
    pub fn load_diff(&mut self, date: NaiveDate, storage: Arc<dyn Storage>) -> Response {
        match self.cache.entry(date) {
            // must be from previous pass due to dedup
            hash_map::Entry::Occupied(v) if v.get().is_loading() => {}
            hash_map::Entry::Occupied(_) => return Response::empty(), // short circuit
            hash_map::Entry::Vacant(v) => {
                v.insert(Resource::Loading);
            }
        }

        Response::future(async move {
            match Self::load(date, storage).await {
                Ok(diff) => Request::RetryWithDeps(vec![RequestType::Fresh(Request::CacheDiff(
                    date,
                    Resource::Loaded(diff),
                ))]),
                Err(err) => Request::RetryWithDeps(vec![RequestType::Fresh(Request::CacheDiff(
                    date,
                    Resource::Failed(err.into()),
                ))]),
            }
        })
    }

    pub fn cache_diff(&mut self, date: NaiveDate, diff: Resource<Diff>) -> Response {
        self.cache.insert(date, diff);
        Response::empty()
    }
}

impl DiffManager {
    pub fn get(&self, date: &NaiveDate) -> Option<&Resource<Diff>> {
        self.cache.get(date)
    }

    pub fn set(&mut self, date: NaiveDate, content: Resource<Diff>) {
        self.cache.insert(date, content);
    }

    pub async fn load(
        date: NaiveDate,
        storage: Arc<dyn Storage>,
    ) -> Result<Diff, Box<dyn Error + Send + Sync>> {
        match storage
            .read("diffs", &format!("{}.diff.json", date.format("%Y-%m-%d")))
            .await?
        {
            Some(bytes) => Ok(serde_json::from_slice(&bytes)?),
            None => Ok(Diff::new(date)),
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
                        "diffs",
                        &format!("{}.diff.json", date.format("%Y-%m-%d")),
                        &bytes,
                    )
                    .await?;
            }
            Some(Resource::Loading) => warn!("Writing diff for {date} but is loading"),
            Some(Resource::Failed(err)) => warn!("Writing diff for {date} but is failed : {err}"),
            None => warn!("Writing diff for {date} but is not in cache"),
        };

        Ok(())
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
/// note: a diff does not have any dependencies
pub struct Diff {
    pub id: NaiveDate,

    // applied in order:
    pub removed_entries: LinkedHashSet<RuleId>,
    // TODO: more diffs e.g. edited items, different sticker
}

impl Diff {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            id: date,
            removed_entries: LinkedHashSet::new(),
        }
    }
}
