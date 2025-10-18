// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/icons.toml
// 0e56c2cec3251ca9aa9d4b8e0fc0776bfe67fc4ced444eee6f00dca6831f4891
use iced::Font;
use iced::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/icons.ttf");

pub fn refresh<'a>() -> Text<'a> {
    icon("\u{E760}")
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
