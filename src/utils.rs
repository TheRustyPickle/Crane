use iced::{Font, font::Weight};

pub fn bold() -> Font {
    Font {
        weight: Weight::Bold,
        ..Default::default()
    }
}
