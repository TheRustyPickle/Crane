use iced::widget::{center, column, container, progress_bar, row, text};
use iced::{Alignment, Color, Element, Length, Theme};

use crate::icon::right_arrow;
use crate::utils::{bold, primary_button, secondary_button};
use crate::{MainWindow, Message, OperationType, Page};

pub const OPERATION_CONTAINER: f64 = 60.0;
pub const OPERATION_CONTAINER_KEY: &str = "operation_container_height";

pub const FETCH_PROGRESS_HEIGHT: f64 = 50.0;
pub const FETCH_PROGRESS_HEIGHT_KEY: &str = "fetch_progress_height";
pub const FETCH_PROGRESS_KEY: &str = "fetch_progress";

pub const OPERATION_PROGRESS_KEY: &str = "operation_progress";

impl MainWindow {
    #[must_use]
    pub fn fetch_progress(&self) -> Element<'_, Message> {
        let total_item = self.crate_list.len();

        let progress_bar = container(
            progress_bar(
                0.0..=100.0,
                self.lerp_state.get(FETCH_PROGRESS_KEY).unwrap_or_default() as f32,
            )
            .girth(5.0),
        )
        .padding(10);

        let item_text = text(format!(
            "Fetching crates ({} of {})",
            self.fetch_progress.unwrap_or_default(),
            total_item
        ))
        .font(bold());

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

    #[must_use]
    pub fn operation_prompt(&self) -> Element<'_, Message> {
        let total_operation = self.delete_crates.len() + self.update_crates.len();

        let operation_text = if let Some(working) = self.operation_crate.as_ref() {
            let operation_text = match working.operation_type {
                OperationType::Update => format!("Updating {}", working.name),
                OperationType::Delete => format!("Deleting {}", working.name),
            };

            text(operation_text).font(bold())
        } else {
            let mut operation_text = format!("{total_operation} pending operations: ");

            let mut parts = vec![];

            if !self.delete_crates.is_empty() {
                parts.push(format!("{} delete", self.delete_crates.len()));
            }

            if !self.update_crates.is_empty() {
                parts.push(format!("{} update", self.update_crates.len()));
            }

            operation_text.push_str(&parts.join(", "));

            text(operation_text).font(bold())
        };

        let mut apply_button = primary_button(text("Apply").font(bold()).style(|_| text::Style {
            color: Some(Color::WHITE),
        }));

        let cancel_button =
            secondary_button(text("Cancel").font(bold())).on_press(Message::CancelOperation);

        let mut log_button = primary_button(right_arrow().style(|_| text::Style {
            color: Some(Color::WHITE),
        }));

        if let Page::Crates = self.showing {
            log_button = log_button.on_press(Message::ShowLog);
        }

        let mut log_button = container(log_button)
            .align_x(Alignment::End)
            .align_y(Alignment::Center);

        if self.fetch_progress.is_none() {
            apply_button = apply_button.on_press(Message::ApplyOperation);
        }

        let mut layout = column![center(operation_text)].width(Length::Fill);

        if self.operation_crate.is_none() {
            let buttons = row![cancel_button, apply_button, log_button]
                .spacing(10)
                .align_y(Alignment::Center);

            let button_container = container(buttons)
                .align_x(Alignment::End)
                .width(Length::Fill)
                .padding(5);

            layout = layout.push(button_container);
        } else {
            let progress_bar = container(
                progress_bar(
                    0.0..=100.0,
                    self.lerp_state
                        .get(OPERATION_PROGRESS_KEY)
                        .unwrap_or_default() as f32,
                )
                .girth(5.0),
            )
            .width(Length::Fill)
            .padding(5);

            log_button = log_button.padding(5);

            let container = row![progress_bar, log_button]
                .spacing(5.0)
                .align_y(Alignment::Center);

            layout = layout.push(container);
        }

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
