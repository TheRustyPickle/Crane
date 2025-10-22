use iced::border::Radius;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text::Wrapping;
use iced::widget::tooltip::Position;
use iced::widget::{center, column, container, mouse_area, row, scrollable, space, text, tooltip};
use iced::{Alignment, Border, Color, Element, Length, Padding, Shadow, Theme};

use crate::icon::{github, refresh, tick, trash};
use crate::utils::{bold, danger_button, primary_button, toggler_button};
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
                } else {
                    for_removal = true;
                }

                let mut icon = if for_removal {
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

                icon = icon.align_x(Alignment::Center);

                let mut icon_button = if for_removal {
                    danger_button(icon)
                } else {
                    primary_button(icon)
                };

                icon_button = icon_button.width(40);

                let mut feature_list = row![].spacing(5);

                let default_active = !crate_item.no_default_features;

                let default_feature = toggler_button(text("default").size(10), default_active)
                    .on_press(Message::FeatureToggle {
                        crate_name: crate_item.name.clone(),
                        feature_name: String::from("default"),
                    });

                feature_list = feature_list.push(default_feature);

                if let Some(crate_response) = &crate_item.crate_response {
                    let crate_name = crate_item.name.clone();
                    if for_removal {
                        icon_button = icon_button.on_press(Message::DeletePressed(crate_name));
                    } else {
                        icon_button = icon_button.on_press(Message::UpdatePressed(crate_name));
                    }

                    let version_data = &crate_response.versions[0];

                    for feature in version_data.features.keys() {
                        if feature == "default" {
                            continue;
                        }

                        let feature_active = crate_item.activated_features.contains(feature);

                        let feature_button = toggler_button(text(feature).size(10), feature_active)
                            .on_press(Message::FeatureToggle {
                                crate_name: crate_item.name.to_string(),
                                feature_name: feature.to_string(),
                            });
                        feature_list = feature_list.push(feature_button);
                    }
                } else {
                    icon_button =
                        icon_button.on_press(Message::DeletePressed(crate_item.name.clone()));
                }

                let feature_layout = scrollable(
                    container(feature_list).width(Length::Fill).height(30),
                )
                .direction(scrollable::Direction::Horizontal(
                    Scrollbar::new().width(5).scroller_width(5),
                ));

                let git_tooltip_content = if let Some(git_link) = &crate_item.git_link {
                    text(git_link)
                } else {
                    text("Enable to use --git flag and install from a git repository")
                };

                let git_button = tooltip(
                    container(
                        toggler_button(
                            github().size(12).align_x(Alignment::Center),
                            crate_item.git_link.is_some(),
                        )
                        .on_press(Message::ToggleGitLink {
                            crate_name: crate_item.name.clone(),
                        })
                        .width(40),
                    )
                    .align_x(Alignment::End),
                    git_tooltip_content,
                    Position::Top,
                )
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(palette.background.weaker.color.into()),
                        text_color: Some(palette.background.weak.text),
                        border: Border {
                            radius: 8.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });

                let actions = column![version_text.size(15), icon_button]
                    .spacing(8)
                    .align_x(Alignment::End);

                let card_content = row![details, actions].spacing(10);

                let card_layout =
                    column![card_content, row![feature_layout, git_button].spacing(5)].spacing(5);

                let card = container(card_layout)
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        let mut background = palette.background.base.color;

                        if let Some(hover_index) = self.hovering
                            && index == hover_index
                        {
                            background = palette.background.weak.color;
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
                    .padding(5);

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
