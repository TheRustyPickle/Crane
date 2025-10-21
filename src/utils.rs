use iced::font::{Family, Weight};
use iced::widget::button::Status;
use iced::widget::{Button, button};
use iced::{Border, Theme};
use iced::{Color, Font};

pub fn bold() -> Font {
    Font {
        weight: Weight::Bold,
        ..Default::default()
    }
}

pub fn mono() -> Font {
    Font {
        family: Family::Monospace,
        ..Default::default()
    }
}

pub fn primary_button<'a, Message>(
    content: impl Into<iced::Element<'a, Message>> + 'a,
) -> Button<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    button(content).style(|theme: &Theme, status| {
        let palette = theme.extended_palette();
        let mut style = button::Style {
            border: Border {
                radius: 8.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        style.background = Some(match status {
            Status::Active => palette.primary.strong.color.into(),
            Status::Hovered => palette.primary.weak.color.into(),
            Status::Pressed => palette.primary.base.color.into(),
            Status::Disabled => palette.background.strongest.color.into(),
        });

        style
    })
}

pub fn secondary_button<'a, Message>(
    content: impl Into<iced::Element<'a, Message>> + 'a,
) -> Button<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    button(content).style(|theme: &Theme, status| {
        let palette = theme.extended_palette();

        let mut style = button::Style {
            border: Border {
                radius: 8.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        style.background = Some(match status {
            Status::Active => palette.background.weak.color.into(),
            Status::Hovered => palette.background.base.color.into(),
            Status::Pressed => palette.background.strong.color.into(),
            Status::Disabled => palette.background.strongest.color.into(),
        });

        style
    })
}

pub fn danger_button<'a, Message>(
    content: impl Into<iced::Element<'a, Message>> + 'a,
) -> Button<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    button(content).style(|theme: &Theme, status| {
        let palette = theme.extended_palette();

        let mut style = button::Style {
            border: Border {
                radius: 8.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        style.background = Some(match status {
            Status::Active => Color::parse("#FFCDD2").unwrap().into(),
            Status::Hovered => Color::parse("#FFB3B8").unwrap().into(),
            Status::Pressed => Color::parse("#FF999D").unwrap().into(),
            Status::Disabled => palette.background.strongest.color.into(),
        });

        style
    })
}
