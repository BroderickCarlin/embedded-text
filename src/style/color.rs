//! Colors.
use embedded_graphics::pixelcolor::{BinaryColor, Rgb555, Rgb565, Rgb888};

/// 24bit RGB color
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Rgb {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl Rgb {
    /// Creates a new color value.
    #[inline]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for BinaryColor {
    #[inline]
    fn from(rgb: Rgb) -> Self {
        if rgb == Rgb::new(255, 255, 255) {
            Self::On
        } else {
            Self::Off
        }
    }
}

impl From<Rgb> for Rgb888 {
    #[inline]
    fn from(rgb: Rgb) -> Self {
        Self::new(rgb.r, rgb.g, rgb.b)
    }
}

impl From<Rgb> for Rgb555 {
    #[inline]
    fn from(rgb: Rgb) -> Self {
        Self::new(rgb.r >> 3, rgb.g >> 3, rgb.b >> 3)
    }
}

impl From<Rgb> for Rgb565 {
    #[inline]
    fn from(rgb: Rgb) -> Self {
        Self::new(rgb.r >> 3, rgb.g >> 2, rgb.b >> 3)
    }
}
