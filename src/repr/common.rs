use std::ops::{Add, Sub};

use iced::{Color, Rectangle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Colour> for iced::Color {
    fn from(other: Colour) -> iced::Color {
        iced::Color {
            r: other.r,
            g: other.g,
            b: other.b,
            a: other.a,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
// a point: normalised with Y between 1 and -1
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn into_raw(&self, bounds: Rectangle) -> (f32, f32) {
        let scaling_factor = bounds.height / 2.0;

        (
            bounds.width / 2.0 + self.x * scaling_factor,
            bounds.height / 2.0 + self.y * scaling_factor,
        )
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add<Vector> for Point {
    type Output = Self;

    fn add(self, vec: Vector) -> Self::Output {
        Self {
            x: self.x + vec.x,
            y: self.y + vec.y,
        }
    }
}

impl Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, vec: Vector) -> Self::Output {
        Self {
            x: self.x - vec.x,
            y: self.y - vec.y,
        }
    }
}

pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn into_raw(&self, bounds: Rectangle) -> (f32, f32) {
        let scaling_factor = bounds.height / 2.0;
        (self.x * scaling_factor, self.y * scaling_factor)
    }
}
