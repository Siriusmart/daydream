use iced::{Color, Point, Rectangle, Renderer, Size, Theme, Vector, mouse, widget::canvas::*};

use crate::{
    app::message::{Request, RequestWrapper},
    repr::{
        day::{Day, DayManager},
        rule::RuleManager,
    },
};

pub struct Board<'a> {
    day: Day,
    rules: &'a RuleManager,
}

impl<'a> Board<'a> {
    pub fn new(day: Day, rules: &'a RuleManager) -> Self {
        Self { day, rules }
    }
}

impl<'a> Program<RequestWrapper> for Board<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        self.day
            .draw(self.rules, renderer, bounds)
            .into_iter()
            .map(|frame| frame.into_geometry())
            .collect()
    }
}
