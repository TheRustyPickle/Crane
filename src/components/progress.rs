use iced::font::Weight;
use iced::widget::{column, container, progress_bar, text};
use iced::{Alignment, Element, Font, Length, Theme};

use crate::{MainWindow, Message};

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

        let container_height = self.lerp_state.get("fetch_progress_height").unwrap_or(0.0) as f32;

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
}
