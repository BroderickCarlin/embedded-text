//! # Example: Scrolling vertical text alignment.
//!
//! This example demonstrates drawing a piece of text using the conventional vertical alignment options.
//! The example uses different, but equivalent ways to specify the alignment options.

use std::convert::Infallible;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use embedded_text::{
    alignment::VerticalAlignment,
    style::{HeightMode, TextBoxStyleBuilder, VerticalOverdraw},
    TextBox,
};

fn main() -> Result<(), Infallible> {
    let mut display = SimulatorDisplay::new(Size::new(128, 96));

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // Divide the screen into 3 64px wide columns.
    TextBox::with_vertical_alignment(
        "Short text using Scrolling alignment is aligned to the top.",
        Rectangle::new(Point::zero(), Size::new(64, 96)),
        character_style,
        VerticalAlignment::Scrolling,
    )
    .draw(&mut display)?;

    TextBox::with_textbox_style(
        "Some longer text to demonstrate that Scrolling alignment aligns text so that the \
        bottom line is always visible.",
        Rectangle::new(Point::new(64, 0), Size::new(64, 96)),
        character_style,
        TextBoxStyleBuilder::new()
            .vertical_alignment(VerticalAlignment::Scrolling)
            .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
            .build(),
    )
    .draw(&mut display)?;

    // Set up the window and show the display's contents.
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .scale(2)
        .build();
    Window::new("Vertical alignment example", &output_settings).show_static(&display);

    Ok(())
}
