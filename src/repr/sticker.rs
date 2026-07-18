use iced::{Renderer, widget::canvas::Frame};
use serde::{Deserialize, Serialize};

use crate::repr::common::Point;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Sticker {
    pub shape: StickerKind,
    pub origin: Point,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum StickerKind {
    #[serde(rename = "rect")]
    Rect,
}
