use std::{
    io::stdin,
    path::Path,
    pin::Pin,
    sync::{Arc, RwLock},
};

use chrono::{Local, NaiveDate};
use iced::{
    Alignment, Element, Length, Task,
    alignment::{Horizontal, Vertical},
    border,
    widget::{Canvas, button, column, container, stack, text},
};
use log::{debug, info};
use task_dag::DedupedTaskDag;

use crate::{
    app::{
        board::Board,
        message::{Request, RequestType, RequestWrapper, Response},
        view_state::ViewState,
    },
    repr::{
        day::DayManager,
        diff::DiffManager,
        resource::Resource,
        rule::RuleManager,
        storage::{FsStorage, Storage},
    },
};

pub struct App {
    storage: Arc<dyn Storage>,
    days: DayManager,
    rules: RuleManager,
    diffs: DiffManager,
    tasks: DedupedTaskDag<Request>,

    view_state: ViewState,
}

impl App {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            storage,
            days: DayManager::default(),
            rules: RuleManager::default(),
            diffs: DiffManager::default(),
            tasks: DedupedTaskDag::new(),

            view_state: ViewState::default(),
        }
    }
}

impl App {
    pub fn update(&mut self, req: RequestWrapper) -> Task<RequestWrapper> {
        info!("{req:?}");

        match req.content.clone() {
            Request::RetryWithDeps(deps) => {
                let response_to = req
                    .response_to
                    .expect("retry with deps must have a response_to");

                self.tasks.mark_parked(response_to.clone());

                for dep in deps {
                    match dep {
                        RequestType::Fresh(dep) => {
                            self.tasks.add_parked_dependencies(&response_to, vec![dep])
                        }
                        RequestType::Any(dep) => {
                            self.tasks.add_any_dependencies(&response_to, vec![dep])
                        }
                    }
                }
            }
            Request::Do(deps) => {
                if let Some(response_to) = req.response_to {
                    self.tasks.mark_done(&response_to);
                }

                for dep in deps {
                    match dep {
                        RequestType::Fresh(dep) => self.tasks.add_parked(dep),
                        RequestType::Any(dep) => self.tasks.add_any(dep),
                    }
                }
            }
            task => {
                if let Some(response_to) = req.response_to {
                    self.tasks.mark_done(&response_to);
                }

                if task != Request::Empty {
                    self.tasks.add_parked(task);
                }
            }
        }

        let doables = self
            .tasks
            .doables()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        doables.iter().for_each(|doable| {
            let doable = (*doable).clone();
            // self.tasks.add_parked(doable.clone());
            self.tasks.mark_running(doable);
        });

        let tasks_todo = doables
            .into_iter()
            .map(|task| match self.handle(task.clone()) {
                Response::_Value(v) => Task::done(RequestWrapper {
                    response_to: Some(task),
                    content: v,
                }),
                Response::_Future(f) => Task::perform(f, |response| RequestWrapper {
                    response_to: Some(task),
                    content: response,
                }),
            })
            .collect::<Vec<_>>();

        Task::batch(tasks_todo)
    }

    /// request -> response
    fn handle(&mut self, request: Request) -> Response {
        match request {
            Request::RetryWithDeps(_) | Request::Do(_) | Request::Empty => {
                unreachable!("disallowed request passed to handle")
            }

            Request::GenerateNewDayRaw(date) => self.days.generate_new_day_raw(date, &self.rules),
            Request::LoadDay(date) => self.days.load_day(date, &self.diffs, &self.rules),
            Request::CacheDay(date, value) => self.days.cache_day(date, value),
            Request::LoadDayRaw(date) => self.days.load_day_raw(date, self.storage.clone()),
            Request::CacheDayRaw(date, value) => self.days.cache_day_raw(date, value),

            Request::LoadDiff(date) => self.diffs.load_diff(date, &self.rules),
            Request::CacheDiff(date, value) => self.diffs.cache_diff(date, value),
            Request::LoadDiffRaw(date) => self.diffs.load_diff_raw(date, self.storage.clone()),
            Request::CacheDiffRaw(date, diff) => self.diffs.cache_diff_raw(date, diff),

            Request::LoadRuleIndex => self.rules.load_rule_index(self.storage.clone()),
            Request::CacheRuleIndex(value) => self.rules.cache_rule_index(value),
            Request::LoadAllRulesUntilInclude(date) => {
                self.rules.load_all_rules_until_include(date)
            }
            Request::LoadRule(id) => self.rules.load_rule(id, self.storage.clone()),
            Request::CacheRule(id, value) => self.rules.cache_rule(id, value),

            Request::ShowDay(date) => self.view_state.show_day(date, &self.days),
        }
    }

    pub fn view(&self) -> Element<'_, RequestWrapper> {
        match self.days.get(&self.view_state.board.date) {
            None => text("not loaded").into(),
            Some(Resource::Loading) => text("loading").into(),
            Some(Resource::Failed(err)) => text!("failed {err:?}").into(),
            Some(Resource::Loaded(day)) => Canvas::new(Board::new(day, &self.rules))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        }
    }
}
