use std::{collections::HashMap, hash::Hash, time::Duration};

use chrono::NaiveDateTime;
use iced::{
    Color, Font, Pixels, Point, Rectangle, Renderer, Size,
    widget::{
        canvas::{self, Frame},
        text::{Ellipsis, Wrapping},
    },
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    macros::simple_id,
    repr::{
        common::Vector,
        rule::{Rule, RuleId},
        sticker::{Sticker, StickerKind},
    },
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Entry {
    // date + id uniquely identifes an entry
    pub id: RuleId,
    pub sticker: Sticker,
    pub sessions: Vec<Interval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ongoing_session: Option<NaiveDateTime>, // starting time
}

impl Entry {
    pub fn draw(
        &self,
        renderer: &Renderer,
        bounds: Rectangle,
        rules: &HashMap<RuleId, Rule>,
    ) -> Frame {
        let mut frame = Frame::new(renderer, bounds.size());

        match self.sticker.shape {
            StickerKind::Memo => {
                let text_bounds = iced::Rectangle::new(
                    iced::Point::from(
                        (self.sticker.origin - Vector::new(0.2, 0.2)).into_raw(bounds),
                    ),
                    iced::Size::from(Vector::new(0.4, 0.4).into_raw(bounds)),
                );

                frame.with_clip(Rectangle::new(Point::ORIGIN, frame.size()), |frame| {
                    frame.fill_rectangle(
                        iced::Point::from(
                            (self.sticker.origin - Vector::new(0.21, 0.21)).into_raw(bounds),
                        ),
                        iced::Size::from(Vector::new(0.42, 0.42).into_raw(bounds)),
                        iced::Color::from(self.sticker.colour.secondary()),
                    );
                    frame.fill_rectangle(
                        text_bounds.position(),
                        text_bounds.size(),
                        iced::Color::from(self.sticker.colour.primary()),
                    );
                });

                frame.with_clip(text_bounds, |frame| {
                    frame.translate(iced::Vector::new(text_bounds.x, text_bounds.y));
                    frame.fill_text(canvas::Text {
                        content: rules
                            .get(&self.id)
                            .expect("rule loaded")
                            .label()
                            .to_string(),
                        position: iced::Point::ORIGIN,
                        max_width: text_bounds.width,
                        size: Pixels::from(text_bounds.height / 6.0),
                        wrapping: Wrapping::Word,
                        ..Default::default()
                    });
                });
            }
        }

        frame
    }
}

impl Entry {
    pub fn new(rule: &Rule) -> Self {
        Self {
            id: rule.id(),
            sticker: rule.default_sticker().clone(),
            sessions: vec![],
            ongoing_session: None,
        }
    }

    pub fn rule_id(&self) -> RuleId {
        self.id
    }

    pub fn elapsed(&self, now: NaiveDateTime) -> Duration {
        self.sessions
            .iter()
            .map(Interval::duration)
            .sum::<Duration>()
            + self
                .ongoing_session
                .map(|start| Interval { start, end: now }.duration())
                .unwrap_or_default()
    }
}

simple_id!(Entry);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Interval {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

impl Interval {
    pub fn duration(&self) -> Duration {
        Duration::from_secs((self.end - self.start).abs().as_seconds_f64() as u64)
    }
}
