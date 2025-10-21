use iced::widget::scrollable::Scrollbar;
use iced::widget::{column, container, row, scrollable, space, text};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::icon::left_arrow;
use crate::utils::{mono, primary_button};
use crate::{MainWindow, Message};

impl MainWindow {
    pub fn log_page(&self) -> Element<'_, Message> {
        let back_button = container(
            primary_button(
                left_arrow()
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Center)
                    .style(|_| text::Style {
                        color: Some(Color::WHITE),
                    }),
            )
            .on_press(Message::ShowCrates)
            .width(60),
        )
        .padding(5);

        let mut log_text = self.logs.join("\n");

        if log_text.is_empty() {
            log_text = "No logs to show".to_string();
        }

        let logs = container(
            text(log_text)
                .wrapping(text::Wrapping::Glyph)
                .font(mono())
                .size(15),
        )
        .padding(5);

        let logs_container = row![logs, space::horizontal()];

        let scroll_area = container(
            scrollable(logs_container).direction(scrollable::Direction::Vertical(Scrollbar::new())),
        )
        .padding(Padding {
            right: 5.0,
            bottom: 5.0,
            ..Default::default()
        })
        .height(Length::Fill);

        column![back_button, scroll_area].spacing(10).into()
    }
}
