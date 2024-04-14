#![allow(unused)]

use iced::{
    border::Radius,
    theme::{self, Text},
    widget::{self, button, image, slider, Image},
    Color, ContentFit, Length, Theme,
};

pub fn background_image<Handle>(handle: impl Into<Handle>) -> Image<Handle> {
    image(handle)
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(ContentFit::Cover)
}

pub fn text(color: Color) -> Text {
    Text::Color(color)
}

pub fn black() -> Color {
    Color::from_rgba8(0, 0, 0, 1.0)
}

pub fn white() -> Color {
    Color::from_rgba8(255, 255, 255, 1.0)
}

pub fn cyan() -> Color {
    Color::from_rgba8(224, 255, 255, 1.0)
}

pub fn blue() -> Color {
    Color::from_rgb8(3, 138, 255)
}

pub struct StyledSlider;

impl slider::StyleSheet for StyledSlider {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (black(), blue()),
                width: 3.0,
                border_radius: Radius::from(0.0),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 10,
                    border_radius: Radius::from(1000.0),
                },
                color: white(),
                border_width: 1.0,
                border_color: black(),
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        Self::active(self, style)
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        Self::active(self, style)
    }
}
