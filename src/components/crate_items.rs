use iced::border::Radius;
use iced::widget::scrollable::Scrollbar;
use iced::widget::text::Wrapping;
use iced::widget::tooltip::Position;
use iced::widget::{center, column, container, mouse_area, row, scrollable, space, text, tooltip};
use iced::{Alignment, Border, Color, Element, Length, Padding, Shadow, Theme};
use std::collections::BTreeSet;

use crate::icon::{github, lock, pin, refresh, tick, trash};
use crate::utils::{bold, danger_button, primary_button, toggler_button, toggler_button_primary};
use crate::{MainWindow, Message};

impl MainWindow {
    #[must_use]
    pub fn crate_items(&self) -> Element<'_, Message> {
        // Don't enable update all button until fetching completes
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

                let mut version_text_string = format!("v{}", crate_item.version);

                if crate_item.git_link.is_some() {
                    if let Some(local_hash) = &crate_item.local_hash
                        && let Some(latest_hash) = &crate_item.latest_hash
                    {
                        let short_local = &local_hash[..local_hash.len().min(5)];
                        let short_latest = &latest_hash[..latest_hash.len().min(5)];

                        if local_hash != latest_hash {
                            version_text_string = format!(
                                "v{} ({} → {})",
                                crate_item.version, short_local, short_latest
                            );
                        } else {
                            version_text_string =
                                format!("v{} ({})", crate_item.version, short_local);
                            for_removal = true;
                        }
                    } else if let Some(local_hash) = &crate_item.local_hash
                        && crate_item.latest_hash.is_none()
                    {
                        let short_local = &local_hash[..local_hash.len().min(5)];

                        version_text_string = format!("v{} ({})", crate_item.version, short_local);
                        for_removal = true;
                    } else if let Some(latest_hash) = &crate_item.latest_hash
                        && crate_item.local_hash.is_none()
                    {
                        let latest_local = &latest_hash[..latest_hash.len().min(5)];

                        version_text_string = format!("v{} → {}", crate_item.version, latest_local);
                    }
                } else if let Some(version) = &crate_item.crates_version {
                    if version > &crate_item.version {
                        version_text_string = format!("v{} → v{}", crate_item.version, version);
                    } else if version == &crate_item.version {
                        for_removal = true;
                    }
                } else {
                    for_removal = true;
                }

                let version_text = text(version_text_string).font(bold());

                if crate_item.pinned {
                    for_removal = true;
                }

                let mut icon = if for_removal {
                    if self.delete_crates.contains_key(&crate_item.name) {
                        tick().color(Color::parse("#F71735").unwrap())
                    } else {
                        trash().color(Color::parse("#F71735").unwrap())
                    }
                } else if self.update_crates.contains_key(&crate_item.name) {
                    tick().color(Color::WHITE)
                } else {
                    refresh().color(Color::WHITE)
                };

                icon = icon.align_x(Alignment::Center);

                let mut icon_button = if for_removal {
                    danger_button(icon)
                } else {
                    primary_button(icon)
                };

                icon_button = icon_button.width(40);

                let crate_name = crate_item.name.clone();
                if for_removal {
                    icon_button = icon_button.on_press(Message::DeletePressed(crate_name));
                } else {
                    icon_button = icon_button.on_press(Message::UpdatePressed(crate_name));
                }

                let mut pin_icon = pin();
                if crate_item.pinned {
                    pin_icon = pin_icon.color(Color::WHITE)
                } else {
                    pin_icon = pin_icon.color(Color::BLACK)
                }

                let pin_button = toggler_button_primary(pin_icon, crate_item.pinned)
                    .on_press(Message::TogglePin(crate_item.name.clone()));

                let mut feature_list = row![].spacing(5);

                // Add the default feature at the start of the row
                let default_active = !crate_item.no_default_features;

                let default_feature = toggler_button(text("default").size(10), default_active)
                    .on_press(Message::FeatureToggle {
                        crate_name: crate_item.name.clone(),
                        feature_name: String::from("default"),
                    });

                feature_list = feature_list.push(default_feature);

                // If crate response is found, list the features gotten from crates.io.
                // If not, if any cached feature list is found, use that.
                if let Some(crate_response) = &crate_item.crate_response {
                    let version_data = &crate_response.versions[0];

                    let sorted_features: BTreeSet<String> =
                        version_data.features.keys().cloned().collect();

                    for feature in sorted_features {
                        if feature == "default" {
                            continue;
                        }

                        let feature_active = crate_item.activated_features.contains(&feature);

                        let feature_button =
                            toggler_button(text(feature.clone()).size(10), feature_active)
                                .on_press(Message::FeatureToggle {
                                    crate_name: crate_item.name.to_string(),
                                    feature_name: feature.to_string(),
                                });
                        feature_list = feature_list.push(feature_button);
                    }
                } else {
                    for feature in &crate_item.cached_features {
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

                let locked_button = tooltip(
                    container(
                        toggler_button(
                            lock().size(12).align_x(Alignment::Center),
                            crate_item.locked,
                        )
                        .on_press(Message::ToggleLocked(crate_item.name.clone())),
                    )
                    .align_x(Alignment::End),
                    "Whether to use --locked flag when installing",
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

                let actions = column![
                    version_text.size(15),
                    row![icon_button, pin_button].spacing(5)
                ]
                .spacing(8)
                .align_x(Alignment::End);

                let card_content = row![details, actions].spacing(10);

                let card_layout = column![
                    card_content,
                    row![feature_layout, git_button, locked_button].spacing(5)
                ]
                .spacing(5);

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
