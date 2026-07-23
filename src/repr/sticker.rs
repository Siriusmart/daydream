use iced::{Renderer, widget::canvas::Frame};
use serde::{Deserialize, Serialize};

use crate::repr::common::{Colour, Point};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Sticker {
    pub shape: StickerKind,
    pub origin: Point,
    pub colour: StickerColours,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StickerColours {
    pub primary: Colour,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<Colour>,
}

impl StickerColours {
    pub fn primary(&self) -> Colour {
        self.primary
    }

    pub fn secondary(&self) -> Colour {
        self.secondary.unwrap_or(Colour {
            r: self.primary.r / 1.5,
            g: self.primary.g / 1.5,
            b: self.primary.b / 1.5,
            a: self.primary.a,
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum StickerKind {
    #[serde(rename = "memo")]
    Memo,
    #[serde(rename = "rect")]
    Rect { width: f32, height: f32 },
}
