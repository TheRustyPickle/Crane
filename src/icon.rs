// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/icons.toml
// 72370b145d74d719770ee2c964dbad71f3bc91aa46700e24a818ed1587108db0
use iced::widget::{text, Text};
use iced::Font;

pub const FONT: &[u8] = include_bytes!("../fonts/icons.ttf");

pub fn refresh<'a>() -> Text<'a> {
    icon("\u{E760}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("icons"))
}
