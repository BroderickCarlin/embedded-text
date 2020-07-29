//! Extends font types with some helper methods.
use embedded_graphics::fonts::Font;

/// `Font` extensions
pub trait FontExt {
    /// Measures a sequence of characters in a line with a determinate maximum width.
    ///
    /// Returns the width of the characters that fit into the given space and whether or not all of
    /// the input fits into the given space.
    fn measure_line(line: &str, max_width: u32) -> LineMeasurement;

    /// Returns the total width of the character plus the character spacing.
    fn total_char_width(c: char) -> u32;

    /// Measure text width
    fn str_width(s: &str) -> u32;
}

/// Result of a `measure_line` function call.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineMeasurement {
    /// The maximum width that still fits into the given width limit.
    pub width: u32,

    /// Whether or not the whole sequence fits into the given width limit.
    pub fits_line: bool,
}

impl LineMeasurement {
    /// Creates a new measurement result object.
    #[inline]
    #[must_use]
    pub const fn new(width: u32, fits_line: bool) -> Self {
        LineMeasurement { width, fits_line }
    }

    /// Creates a new measurement result object for an empty line.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self::new(0, true)
    }
}

impl<F> FontExt for F
where
    F: Font,
{
    #[inline]
    #[must_use]
    fn measure_line(line: &str, max_width: u32) -> LineMeasurement {
        let mut total_width = 0;

        for c in line.chars() {
            let new_width = total_width + F::total_char_width(c);
            if new_width > max_width {
                return LineMeasurement::new(total_width, false);
            } else {
                total_width = new_width;
            }
        }

        LineMeasurement::new(total_width, true)
    }

    #[inline]
    fn total_char_width(c: char) -> u32 {
        F::char_width(c) + F::CHARACTER_SPACING
    }

    #[inline]
    fn str_width(s: &str) -> u32 {
        s.chars().map(F::total_char_width).sum::<u32>()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use embedded_graphics::fonts::Font6x8;

    #[test]
    fn test_max_fitting_empty() {
        assert_eq!(Font6x8::measure_line("", 54), LineMeasurement::new(0, true))
    }

    #[test]
    fn test_max_fitting_exact() {
        let measurement = Font6x8::measure_line("somereall", 54);
        assert_eq!(measurement, LineMeasurement::new(54, true));
    }

    #[test]
    fn test_max_fitting_long_exact() {
        let measurement = Font6x8::measure_line("somereallylongword", 54);
        assert_eq!(measurement, LineMeasurement::new(54, false));
    }

    #[test]
    fn test_max_fitting_long() {
        let measurement = Font6x8::measure_line("somereallylongword", 55);
        assert_eq!(measurement, LineMeasurement::new(54, false));
    }
}
