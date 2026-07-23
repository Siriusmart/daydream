use std::collections::HashMap;

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

impl<'a> Program<Request> for Board {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Request>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                let (mouse::Cursor::Available(point) | mouse::Cursor::Levitating(point)) = cursor
                else {
                    return None;
                };

                Some(Action::publish(Request::CreateRule(
                    Rule::new(self.day.id()).with_default_sticker(Sticker {
                        shape: StickerKind::Memo,
                        origin: Point::from_raw(point, bounds),
                        colour: StickerColours {
                            primary: Colour {
                                r: 1.0,
                                g: 1.0,
                                b: 0.0,
                                a: 1.0,
                            },
                            secondary: None,
                        },
                    }),
                )))
            }
            _ => None,
        }
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
