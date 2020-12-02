use sdl2::pixels::Color;
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
pub struct Spectrum {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

pub static BLACK: Spectrum = Spectrum {
    r: 0.0,
    g: 0.0,
    b: 0.0,
};

impl Spectrum {
    pub fn scale(&self, s: impl Into<f64>) -> Spectrum {
        let s = s.into();
        Spectrum {
            r: self.r * s,
            g: self.g * s,
            b: self.b * s,
        }
    }

    pub fn to_color(&self) -> Color {
        Color::RGB(
            (self.r * 255.0).min(255.0) as u8,
            (self.g * 255.0).min(255.0) as u8,
            (self.b * 255.0).min(255.0) as u8,
        )
    }
}

impl Add for Spectrum {
    type Output = Spectrum;

    fn add(self, other: Self) -> Self::Output {
        Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl Mul for Spectrum {
    type Output = Spectrum;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
        }
    }
}
