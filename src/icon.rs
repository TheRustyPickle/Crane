// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/icons.toml
// a40fdeaf3c83da394db1b15a33c72a8d94791088d09236e285cd6680514b96e7
use iced::widget::{text, Text};
use iced::Font;

pub const FONT: &[u8] = include_bytes!("../fonts/icons.ttf");

pub fn github<'a>() -> Text<'a> {
    icon("\u{F307}")
}

pub fn left_arrow<'a>() -> Text<'a> {
    icon("\u{E00D}")
}

pub fn lock<'a>() -> Text<'a> {
    icon("\u{E096}")
}

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
