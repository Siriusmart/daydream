use std::collections::HashMap;

use chrono::{Local, NaiveDate};

use crate::{
    app::{
        board::Board,
        message::{Request, RequestType, Response},
    },
    repr::{day::DayManager, resource::Resource},
};

impl ViewState {
    pub fn show_day(&mut self, date: NaiveDate, days: &DayManager) -> Response {
        match days.get(&date) {
            Some(Resource::Failed(_)) => todo!(),
            Some(Resource::Loading) | None => {
                Response::value(Request::RetryWithDeps(vec![RequestType::Any(
                    Request::LoadDay(date),
                )]))
            }
            Some(Resource::Loaded(_)) => {
                self.board.date = date;
                Response::empty()
            }
        }
    }
}

#[derive(Default)]
pub struct ViewState {
    pub board: BoardViewState,
}

pub struct BoardViewState {
    pub date: NaiveDate,
    pub boards: HashMap<NaiveDate, Board>,
}

impl Default for BoardViewState {
    fn default() -> Self {
        Self {
            date: Local::now().date_naive(),
            boards: HashMap::new(),
        }
    }
}
