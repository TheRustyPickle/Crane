use iced::font::{Family, Weight};
use iced::widget::button::Status;
use iced::widget::{Button, button, center, container, mouse_area, opaque, stack};
use iced::{Border, Color, Element, Font, Theme};

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
        let palette = theme.palette();
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
        let palette = theme.palette();

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
        let palette = theme.palette();

        let mut style = button::Style {
            border: Border {
                radius: 8.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        style.background = Some(match status {
            Status::Active => "#FFCDD2".parse::<Color>().unwrap().into(),
            Status::Hovered => "#FFB3B8".parse::<Color>().unwrap().into(),
            Status::Pressed => "#FF999D".parse::<Color>().unwrap().into(),
            Status::Disabled => palette.background.strongest.color.into(),
        });

        style
    })
}

pub fn toggler_button<'a, Message>(
    content: impl Into<iced::Element<'a, Message>> + 'a,
    active: bool,
) -> Button<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    button(content).style(move |theme: &Theme, status| {
        let palette = theme.palette();

        let success_base = "#A5D6A7".parse::<Color>().unwrap();
        let success_hover = "#81C784".parse::<Color>().unwrap();
        let success_strong = "#66BB6A".parse::<Color>().unwrap();

        let mut style = button::Style {
            border: Border {
                radius: 8.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        let base_color = if active {
            success_base
        } else {
            palette.background.stronger.color
        };

        style.background = Some(match status {
            Status::Active => base_color.into(),
            Status::Hovered => {
                if active {
                    success_hover.into()
                } else {
                    palette.background.base.color.into()
                }
            }
            Status::Pressed => {
                if active {
                    success_strong.into()
                } else {
                    palette.background.strong.color.into()
                }
            }
            Status::Disabled => palette.background.strongest.color.into(),
        });

        style
    })
}

pub fn toggler_button_primary<'a, Message>(
    content: impl Into<iced::Element<'a, Message>> + 'a,
    active: bool,
) -> Button<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    button(content).style(move |theme: &Theme, status| {
        let palette = theme.palette();

        let mut style = button::Style {
            border: Border {
                radius: 8.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        let base_color = if active {
            palette.primary.strong.color
        } else {
            palette.background.stronger.color
        };

        style.background = Some(match status {
            Status::Active => base_color.into(),
            Status::Hovered => {
                if active {
                    palette.primary.weak.color.into()
                } else {
                    palette.background.base.color.into()
                }
            }
            Status::Pressed => {
                if active {
                    palette.primary.base.color.into()
                } else {
                    palette.background.strong.color.into()
                }
            }
            Status::Disabled => palette.background.strongest.color.into(),
        });

        style
    })
}

pub fn parse_git_link(link: &str) -> Option<(String, String)> {
    if link.starts_with("(git+") {
        let stripped_text = link.strip_prefix("(git+")?.strip_suffix(")")?;

        stripped_text.split_once('#').map(|(git_link, hash)| {
            let split_git_link = git_link.split_once('?').unwrap_or((git_link, ""));

            (split_git_link.0.to_string(), hash.to_string())
        })
    } else {
        None
    }
}

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
