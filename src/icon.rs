// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/icons.toml
// 244f4a7eb0319f009cc94857a20f10eaa01a0c6690f3c452871d6bcc01ec5ed8
use iced::Font;
use iced::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/icons.ttf");

pub fn github<'a>() -> Text<'a> {
    icon("\u{F307}")
}

pub fn left_arrow<'a>() -> Text<'a> {
    icon("\u{E00D}")
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
