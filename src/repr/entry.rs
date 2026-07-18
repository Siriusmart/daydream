use std::{hash::Hash, time::Duration};

use chrono::NaiveDateTime;
use iced::{Color, Point, Rectangle, Renderer, Size, Vector, widget::canvas::Frame};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    macros::simple_id,
    repr::{
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
    pub fn draw(&self, renderer: &Renderer, bounds: Rectangle) -> Frame {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.translate(Vector {
            x: bounds.width / 2.0 + self.sticker.origin.x,
            y: bounds.height / 2.0 + self.sticker.origin.y,
        });

        match self.sticker.shape {
            StickerKind::Rect => {
                frame.fill_rectangle(
                    Point {
                        x: -100.0,
                        y: -100.0,
                    },
                    Size {
                        width: 200.0,
                        height: 200.0,
                    },
                    Color::from_rgb(1.0, 0.0, 0.0),
                );
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
