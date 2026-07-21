use std::collections::HashMap;

use iced::{Color, Point, Rectangle, Renderer, Size, Theme, Vector, mouse, widget::canvas::*};

use crate::{
    app::message::{Request, RequestWrapper},
    repr::{
        day::{Day, DayManager},
        rule::{Rule, RuleId, RuleManager},
    },
};

pub enum BoardState {
    Rect { start: Point },
}

pub struct Board {
    day: Day,
    rules: HashMap<RuleId, Rule>,
}

impl Board {
    pub fn new(day: Day, rules: &RuleManager) -> Self {
        Self {
            rules: day.extract_rules(rules),
            day,
        }
    }
}

impl<'a> Program<RequestWrapper> for Board {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<RequestWrapper>> {
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        self.day
            .draw(&self.rules, renderer, bounds)
            .into_iter()
            .map(|frame| frame.into_geometry())
            .collect()
    }
}
