use iced::font::Weight;
use iced::widget::button::Status;
use iced::widget::{button, center, column, container, progress_bar, row, text};
use iced::{Alignment, Border, Color, Element, Font, Length, Theme};

use crate::{MainWindow, Message};

pub const OPERATION_CONTAINER: f64 = 60.0;
pub const OPERATION_CONTAINER_KEY: &str = "operation_container_height";

pub const FETCH_PROGRESS_HEIGHT: f64 = 50.0;
pub const FETCH_PROGRESS_HEIGHT_KEY: &str = "fetch_progress_height";
pub const PROGRESS_KEY: &str = "fetch_progress";

impl MainWindow {
    pub fn fetch_progress(&self) -> Element<'_, Message> {
        let bold_font = Font {
            weight: Weight::Bold,
            ..Default::default()
        };

        let total_item = self.crate_list.len();

        let progress_bar = container(
            progress_bar(
                0.0..=100.0,
                self.lerp_state.get("fetch_progress").unwrap_or_default() as f32,
            )
            .girth(5.0),
        )
        .padding(10);
        let item_text = text(format!(
            "Fetching crates ({} of {})",
            self.fetch_progress, total_item
        ))
        .font(bold_font);

        let progress = column![item_text, progress_bar]
            .spacing(4)
            .align_x(Alignment::Center);

        let container_height = self
            .lerp_state
            .get(FETCH_PROGRESS_HEIGHT_KEY)
            .unwrap_or(0.0) as f32;

        container(progress)
            .height(Length::Fixed(container_height))
            .style(|them: &Theme| {
                let palette = them.extended_palette();

                container::Style {
                    background: Some(palette.background.weak.color.into()),
                    ..Default::default()
                }
            })
            .into()
    }

    pub fn selected_prompt(&self) -> Element<'_, Message> {
        let bold_font = Font {
            weight: Weight::Bold,
            ..Default::default()
        };

        let mut pending_text = format!(
            "{} pending operations: ",
            self.delete_crates.len() + self.update_crates.len()
        );

        let mut parts = vec![];

        if !self.delete_crates.is_empty() {
            parts.push(format!("{} delete", self.delete_crates.len()));
        }

        if !self.update_crates.is_empty() {
            parts.push(format!("{} update", self.update_crates.len()));
        }

        pending_text.push_str(&parts.join(", "));

        let pending_operation = text(pending_text).font(bold_font);

        let apply_button = button(text("Apply").font(bold_font).style(|_| text::Style {
            color: Some(Color::WHITE),
        }))
        .on_press(Message::ApplyOperation)
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();
            let mut style = button::Style {
                border: Border {
                    radius: 8.into(),
                    ..Default::default()
                },
                ..Default::default()
            };
            match status {
                Status::Active => {
                    style.background = Some(palette.primary.strong.color.into());
                }
                Status::Hovered => {
                    style.background = Some(palette.primary.weak.color.into());
                }
                Status::Pressed => {
                    style.background = Some(palette.primary.base.color.into());
                }
                Status::Disabled => {
                    style.background = Some(palette.background.strongest.color.into());
                }
            }

            style
        });

        let cancel_button = button(text("Cancel").font(bold_font))
            .on_press(Message::CancelOperation)
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();

                let mut style = button::Style {
                    border: Border {
                        radius: 8.into(),
                        width: 1.0,
                        color: palette.background.strong.color,
                    },
                    ..Default::default()
                };

                match status {
                    Status::Active => {
                        style.background = Some(palette.background.weak.color.into());
                    }
                    Status::Hovered => {
                        style.background = Some(palette.background.base.color.into());
                    }
                    Status::Pressed => {
                        style.background = Some(palette.background.strong.color.into());
                    }
                    Status::Disabled => {
                        style.background = Some(palette.background.strongest.color.into());
                        style.border.color = palette.background.strongest.color;
                    }
                }

                style
            });

        let buttons = row![cancel_button, apply_button]
            .spacing(10)
            .align_y(Alignment::Center);

        let button_container = container(buttons)
            .align_x(Alignment::End)
            .width(Length::Fill)
            .padding(5);

        let layout = column![center(pending_operation), button_container].width(Length::Fill);

        let container_height = self.lerp_state.get(OPERATION_CONTAINER_KEY).unwrap_or(0.0) as f32;

        container(layout)
            .height(Length::Fixed(container_height))
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(palette.background.weak.color.into()),
                    ..Default::default()
                }
            })
            .into()
    }
}
