use iced::widget::text_input::Status;
use iced::widget::{column, container, row, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Theme};

use crate::message::GitInputEvent;
use crate::utils::{bold, danger_button, primary_button};
use crate::{MainWindow, Message};

impl MainWindow {
    #[must_use]
    pub fn git_modal(&self) -> Element<'_, Message> {
        container(
            column![
                column![
                    column![
                        text("Crate Git Link").size(20),
                        text_input(
                            "https://github.com/iced-rs/iced",
                            &self.git_input.modal_text
                        )
                        .on_input(|text| Message::GitInput(GitInputEvent::Input(text)))
                        .on_submit(Message::GitInput(GitInputEvent::Submit))
                        .padding(5)
                        .style(|theme: &Theme, status| {
                            let palette = theme.extended_palette();

                            let active = text_input::Style {
                                background: (palette.background.base.color).into(),
                                border: Border {
                                    radius: 8.0.into(),
                                    width: 1.0,
                                    color: palette.background.strong.color,
                                },
                                icon: palette.background.weak.text,
                                placeholder: palette.secondary.base.color,
                                value: palette.background.base.text,
                                selection: palette.primary.weak.color,
                            };

                            match status {
                                Status::Active => active,
                                Status::Hovered => text_input::Style {
                                    border: Border {
                                        color: palette.background.base.text,
                                        ..active.border
                                    },
                                    ..active
                                },
                                Status::Focused { .. } => text_input::Style {
                                    border: Border {
                                        color: palette.primary.strong.color,
                                        ..active.border
                                    },
                                    ..active
                                },
                                Status::Disabled => text_input::Style {
                                    background: (palette.background.weak.color).into(),
                                    value: active.placeholder,
                                    placeholder: palette.background.strongest.color,
                                    ..active
                                },
                            }
                        }),
                    ]
                    .spacing(5),
                    container(
                        row![
                            primary_button(
                                text("Submit")
                                    .color(Color::WHITE)
                                    .font(bold())
                                    .align_x(Alignment::Center)
                            )
                            .on_press(Message::GitInput(GitInputEvent::Submit))
                            .width(Length::Fill),
                            danger_button(text("Cancel").font(bold()).align_x(Alignment::Center))
                                .on_press(Message::GitInput(GitInputEvent::HideModal))
                                .width(Length::Fill),
                        ]
                        .spacing(5)
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                ]
                .spacing(10)
            ]
            .spacing(20),
        )
        .width(500)
        .padding(10)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: Some(palette.background.weak.text),
                border: Border {
                    radius: 8.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
    }
}
