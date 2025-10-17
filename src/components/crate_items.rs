use iced::border::Radius;
use iced::widget::button::Status;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text::Wrapping;
use iced::widget::{button, center, column, container, row, scrollable, space, text};
use iced::{Alignment, Border, Color, Element, Theme};

use crate::icon::{refresh};
use crate::{MainWindow, Message};

impl MainWindow {
    pub fn crate_items(&self) -> Element<'_, Message> {
        let crate_cards = self
            .crate_list
            .iter()
            .enumerate()
            .fold(column![], |column, (index, crate_item)| {
                let details = column![
                    text(&crate_item.name).size(22),
                    text(&crate_item.description)
                        .size(16)
                        .wrapping(Wrapping::Glyph),
                    space::horizontal(),
                ]
                .spacing(5)
                .align_x(Alignment::Start);

                let actions = column![
                    text(format!("v{}", crate_item.version)).size(15),
                    button(refresh())
                        .on_press(Message::UpdatePressed(crate_item.name.clone()))
                        .style(|theme: &Theme, status| {
                            let palette = theme.extended_palette();
                            let mut style = button::Style {
                                border: Border {
                                    color: palette.primary.strong.color,
                                    width: 1.0,
                                    radius: 10.into(),
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
                                    style.shadow.offset = [1.0, 1.0].into();
                                }
                                Status::Disabled => {
                                    style.background = Some(palette.background.weak.color.into());
                                }
                            }

                            style
                        })
                ]
                .spacing(8)
                .align_x(Alignment::End);

                let card_content = row![details, actions].spacing(10);

                let card = container(card_content)
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        container::Style {
                            background: Some(palette.background.weak.color.into()),
                            border: Border {
                                color: Color::BLACK,
                                width: 0.5,
                                radius: Radius {
                                    top_left: if index == 0 { 10.0 } else { 0.0 },
                                    top_right: if index == 0 { 10.0 } else { 0.0 },
                                    bottom_left: if index == self.crate_list.len() - 1 {
                                        10.0
                                    } else {
                                        0.0
                                    },
                                    bottom_right: if index == self.crate_list.len() - 1 {
                                        10.0
                                    } else {
                                        0.0
                                    },
                                },
                            },
                            ..Default::default()
                        }
                    })
                    .padding(10);

                column.push(card)
            })
            .padding(20)
            .max_width(800.0);

        container(
            scrollable(center(crate_cards))
                .direction(scrollable::Direction::Vertical(Scrollbar::new())),
        )
        .padding(10)
        .into()
    }
}
