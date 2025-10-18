use iced::border::Radius;
use iced::font::Weight;
use iced::widget::button::Status;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text::Wrapping;
use iced::widget::{button, center, column, container, mouse_area, row, scrollable, space, text};
use iced::{Alignment, Border, Color, Element, Font, Length, Padding, Shadow, Theme};

use crate::icon::refresh;
use crate::{MainWindow, Message};

impl MainWindow {
    #[must_use]
    pub fn crate_items(&self) -> Element<'_, Message> {
        let bold_font = Font {
            weight: Weight::Bold,
            ..Default::default()
        };

        let crate_cards = self
            .crate_list
            .values()
            .enumerate()
            .fold(column![], |column, (index, crate_item)| {
                let details = column![
                    text(&crate_item.name).size(18).font(bold_font),
                    text(&crate_item.description)
                        .size(15)
                        .wrapping(Wrapping::Glyph),
                    space::horizontal(),
                ]
                .spacing(8)
                .align_x(Alignment::Start);

                let version_text = if let Some(version) = &crate_item.crates_version {
                    if version > &crate_item.version {
                        text(format!("v{} -> v{}", crate_item.version, version))
                    } else {
                        text(format!("v{}", crate_item.version))
                    }
                } else {
                    text(format!("v{}", crate_item.version))
                };

                let actions = column![
                    version_text.size(15),
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
                        let mut background = palette.background.base.color;

                        if let Some(hover_index) = self.hovering
                            && index == hover_index
                        {
                            background = palette.background.weak.color
                        }

                        container::Style {
                            background: Some(background.into()),
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
                            shadow: Shadow {
                                color: Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.35,
                                },
                                offset: [0.0, 0.0].into(),
                                blur_radius: 4.0,
                            },
                            ..Default::default()
                        }
                    })
                    .padding(10);

                let mouse_area = mouse_area(card)
                    .on_enter(Message::Hovering(index))
                    .on_exit(Message::HoveringExit(index));

                column.push(mouse_area)
            })
            .padding(20)
            .max_width(800.0);

        container(
            scrollable(center(crate_cards))
                .direction(scrollable::Direction::Vertical(Scrollbar::new())),
        )
        .padding(Padding {
            right: 5.0,
            ..Default::default()
        })
        .height(Length::Fill)
        .into()
    }
}
