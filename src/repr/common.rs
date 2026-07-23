use std::ops::{Add, Div, Sub};

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

    pub fn from_raw(point: iced::Point, bounds: Rectangle) -> Self {
        let scaling_factor = bounds.height / 2.0;

        Self {
            x: (point.x - bounds.width / 2.0) / scaling_factor,
            y: (point.y - bounds.height / 2.0) / scaling_factor,
        }
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn average(points: &[Point]) -> Self {
        Self {
            x: points.iter().map(|point| point.x).sum::<f32>() / (points.len() as f32),
            y: points.iter().map(|point| point.y).sum::<f32>() / (points.len() as f32),
        }
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

impl Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Self::Output {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Div<f32> for Vector {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Add<Self> for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: rhs.x + self.x,
            y: rhs.y + self.y,
        }
    }
}

impl Vector {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn into_raw(&self, bounds: Rectangle) -> (f32, f32) {
        let scaling_factor = bounds.height / 2.0;
        (self.x * scaling_factor, self.y * scaling_factor)
    }

    pub fn abs_component(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
}
