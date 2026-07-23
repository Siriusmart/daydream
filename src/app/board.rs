use std::collections::HashMap;

use chrono::Local;
use iced::{
    Rectangle, Renderer, Size, Theme, Vector,
    mouse::{self, Button},
    widget::canvas::*,
};

use crate::{
    app::message::{Request, RequestWrapper},
    repr::{
        common::{Colour, Point},
        day::{Day, DayManager},
        entry::Entry,
        rule::{Rule, RuleId, RuleManager},
        sticker::{Sticker, StickerColours, StickerKind},
    },
};

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

pub enum BoardState {
    None,
    Rect1(Point),
}

impl Default for BoardState {
    fn default() -> Self {
        Self::None
    }
}

macro_rules! cursor_pos_or_none {
    ($x: ident, $y: ident) => {
        let (mouse::Cursor::Available($y) | mouse::Cursor::Levitating($y)) = $x else {
            return None;
        };
    };
}

fn rect_from_points(start: Point, end: Point) -> Sticker {
    let size = (end - start).abs_component();

    Sticker {
        shape: StickerKind::Rect {
            width: size.x,
            height: size.y,
        },
        origin: Point::average(&[start, end]),
        colour: StickerColours {
            primary: Colour {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            secondary: None,
        },
    }
}

impl<'a> Program<Request> for Board {
    type State = BoardState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Request>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                cursor_pos_or_none!(cursor, point);
                *state = BoardState::Rect1(Point::from_raw(point, bounds));
                None
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
                if matches!(state, BoardState::Rect1(_)) =>
            {
                Some(Action::request_redraw())
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left))
                if let BoardState::Rect1(start) = state =>
            {
                let start = *start;
                cursor_pos_or_none!(cursor, point);
                let end = Point::from_raw(point, bounds);
                *state = BoardState::None;
                Some(Action::publish(Request::CreateRule(
                    Rule::new(self.day.id()).with_default_sticker(rect_from_points(start, end)),
                )))
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut geometry: Vec<Geometry> = self
            .day
            .draw(&self.rules, renderer, bounds)
            .into_iter()
            .map(|frame| frame.into_geometry())
            .collect();

        match state {
            BoardState::Rect1(start)
                if let mouse::Cursor::Available(point) | mouse::Cursor::Levitating(point) =
                    cursor =>
            {
                let end = Point::from_raw(point, bounds);
                let sticker = rect_from_points(*start, end);
                let fake_rule = Rule::new(Local::now().date_naive()).with_default_sticker(sticker);
                let rules = HashMap::from_iter([(fake_rule.id(), fake_rule.clone())]);
                geometry.push(
                    Entry::new(&fake_rule)
                        .draw(renderer, bounds, &rules)
                        .into_geometry(),
                );
            }
            _ => {}
        }

        geometry
    }
}
