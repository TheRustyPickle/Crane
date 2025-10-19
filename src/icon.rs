// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/icons.toml
// e9bd628ccf279674ed020b121d51bc8c6b86a6f0d9df3fb0e0e47e12ce5e16ae
use iced::Font;
use iced::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/icons.ttf");

pub fn refresh<'a>() -> Text<'a> {
    icon("\u{E760}")
}

pub fn right_arrow<'a>() -> Text<'a> {
    icon("\u{E01A}")
}

pub fn tick<'a>() -> Text<'a> {
    icon("\u{2713}")
}

pub fn trash<'a>() -> Text<'a> {
    icon("\u{E729}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("icons"))
}
