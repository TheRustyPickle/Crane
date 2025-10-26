// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/icons.toml
// 9b157dfbe31ea87e12a0c1bc1b85febe4150d2e873a186d1d80d8ea205cd1872
use iced::Font;
use iced::widget::{Text, text};

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

pub fn pin<'a>() -> Text<'a> {
    icon("\u{E0BA}")
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
