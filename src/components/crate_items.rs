use iced::border::Radius;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text::Wrapping;
use iced::widget::{center, column, container, mouse_area, row, scrollable, space, text};
use iced::{Alignment, Border, Color, Element, Length, Padding, Shadow, Theme};

use crate::icon::{refresh, tick, trash};
use crate::utils::{bold, danger_button, primary_button};
use crate::{MainWindow, Message};

impl MainWindow {
    #[must_use]
    pub fn crate_items(&self) -> Element<'_, Message> {
        let mut select_all_button =
            primary_button(text("Update All").font(bold()).style(|_| text::Style {
                color: Some(Color::WHITE),
            }));

        if self.fetch_progress.is_none() {
            select_all_button = select_all_button.on_press(Message::UpdateAll);
        }

        let button_container = container(select_all_button).padding(Padding {
            bottom: 5.0,
            ..Default::default()
        });

        let crate_columns = column![button_container];

        let crate_cards = self
            .crate_list
            .values()
            .enumerate()
            .fold(crate_columns, |column, (index, crate_item)| {
                let details = column![
                    text(&crate_item.name).size(18).font(bold()),
                    text(&crate_item.description)
                        .size(15)
                        .wrapping(Wrapping::Glyph),
                    space::horizontal(),
                ]
                .spacing(8)
                .align_x(Alignment::Start);

                let mut for_removal = false;

                let mut version_text = text(format!("v{}", crate_item.version)).font(bold());

                if let Some(version) = &crate_item.crates_version {
                    if version > &crate_item.version {
                        version_text =
                            text(format!("v{} → v{}", crate_item.version, version)).font(bold());
                    } else if version == &crate_item.version {
                        for_removal = true;
                    }
                }

                let icon = if for_removal {
                    if self.delete_crates.contains_key(&crate_item.name) {
                        tick().style(|_| text::Style {
                            color: Color::parse("#F71735"),
                        })
                    } else {
                        trash().style(|_| text::Style {
                            color: Color::parse("#F71735"),
                        })
                    }
                } else if self.update_crates.contains_key(&crate_item.name) {
                    tick().style(|_| text::Style {
                        color: Some(Color::WHITE),
                    })
                } else {
                    refresh().style(|_| text::Style {
                        color: Some(Color::WHITE),
                    })
                };

                let mut icon_button = if for_removal {
                    danger_button(icon)
                } else {
                    primary_button(icon)
                };

                if crate_item.crates_version.is_some() {
                    let crate_name = crate_item.name.clone();
                    if for_removal {
                        icon_button = icon_button.on_press(Message::DeletePressed(crate_name))
                    } else {
                        icon_button = icon_button.on_press(Message::UpdatePressed(crate_name))
                    }
                }

                let actions = column![version_text.size(15), icon_button]
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
            bottom: 5.0,
            top: 5.0,
            ..Default::default()
        })
        .height(Length::Fill)
        .into()
    }
}
