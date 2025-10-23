use iced::Task;
use iced::futures::SinkExt;
use log::{error, info};
use semver::Version;

use crate::components::{FETCH_PROGRESS_HEIGHT, FETCH_PROGRESS_HEIGHT_KEY, FETCH_PROGRESS_KEY};
use crate::worker::{WorkerEvent, WorkerInput};
use crate::{MainWindow, OperationCrate, OperationType, Page};

#[derive(Debug, Clone, Default)]
pub struct GitInputState {
    pub modal_text: String,
    pub show_modal: bool,
    crate_name: String,
}

#[derive(Debug, Clone)]
pub enum GitInputEvent {
    HideModal,
    Submit,
    Input(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdatePressed(String),
    DeletePressed(String),
    FetchEvent(WorkerEvent),
    Hovering(usize),
    HoveringExit(usize),
    Tick,
    CancelOperation,
    ApplyOperation,
    ShowLog,
    ShowCrates,
    UpdateAll,
    FeatureToggle {
        crate_name: String,
        feature_name: String,
    },
    ToggleGitLink {
        crate_name: String,
    },
    ToggleLock(String),
    GitInput(GitInputEvent),
    None,
}

impl MainWindow {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdatePressed(crate_name) => {
                let target_crate = self.crate_list.get(&crate_name).unwrap().clone();

                if self.update_crates.contains_key(&crate_name) {
                    self.update_crates.remove(&crate_name);
                } else {
                    self.update_crates.insert(crate_name.clone(), target_crate);
                }

                self.update_lerp_states_operation_container();
            }
            Message::DeletePressed(crate_name) => {
                let target_crate = self.crate_list.get(&crate_name).unwrap().clone();

                if self.delete_crates.contains_key(&crate_name) {
                    self.delete_crates.remove(&crate_name);
                } else {
                    self.delete_crates.insert(crate_name.clone(), target_crate);
                }

                self.update_lerp_states_operation_container();
            }
            Message::FetchEvent(event) => match event {
                WorkerEvent::Ready(mut sender) => {
                    self.worker = Some(sender.clone());
                    let crate_names = self.crate_list.keys().cloned().collect();

                    let rate_limit = self
                        .config
                        .as_ref()
                        .map(|c| c.crate_rate_limit_ms)
                        .unwrap_or(1000);

                    return Task::perform(
                        async move {
                            let _ = sender
                                .send(WorkerInput::GetCrateVersion(crate_names, rate_limit))
                                .await;
                        },
                        |()| Message::None,
                    );
                }
                WorkerEvent::SuccessCrate((details, index)) => {
                    self.fetch_progress = Some(index + 1);

                    let mut progress_status = 0.0;
                    let total_item = self.crate_list.len();

                    if let Some(progress) = self.fetch_progress {
                        progress_status = (progress as f32 / total_item as f32) * 100.0;
                    }

                    self.lerp_state
                        .lerp(FETCH_PROGRESS_KEY, f64::from(progress_status));

                    self.lerp_state
                        .lerp(FETCH_PROGRESS_HEIGHT_KEY, FETCH_PROGRESS_HEIGHT);

                    let description = details
                        .crate_data
                        .description
                        .clone()
                        .unwrap_or(String::from("The crate has no description"));

                    let latest_version = Version::parse(&details.crate_data.max_version);

                    if let Err(e) = latest_version {
                        error!(
                            "Failed to parse version: {e}. Was parsing {}",
                            details.crate_data.max_version
                        );
                        return Task::none();
                    }

                    let latest_version = latest_version.unwrap();

                    let target_crate = self.crate_list.get_mut(&details.crate_data.name).unwrap();

                    self.delete_crates.remove(&details.crate_data.name);

                    if let Some(config) = &mut self.config {
                        let version_data = &details.versions[0];
                        let feature_list = version_data.features.keys().cloned().collect();

                        config.update_cache(
                            details.crate_data.name.clone(),
                            description.clone(),
                            feature_list,
                            latest_version.to_string(),
                        )
                    }

                    target_crate.description = description;
                    target_crate.crates_version = Some(latest_version);
                    target_crate.crate_response = Some(*details);
                }
                WorkerEvent::ErrorCrate(index) => {
                    self.fetch_progress = Some(index + 1);

                    let mut progress_status = 0.0;
                    let total_item = self.crate_list.len();

                    if let Some(progress) = self.fetch_progress {
                        progress_status = (progress as f32 / total_item as f32) * 100.0;
                    }

                    self.lerp_state
                        .lerp(FETCH_PROGRESS_KEY, f64::from(progress_status));

                    self.lerp_state
                        .lerp(FETCH_PROGRESS_HEIGHT_KEY, FETCH_PROGRESS_HEIGHT);
                }
                WorkerEvent::DoneCrateCheck => {
                    self.fetch_progress = None;
                    self.lerp_state.lerp("fetch_progress_height", 0.0);
                }
                WorkerEvent::Log(log) => {
                    self.logs.push(log);

                    if self.logs.len() > 1000 {
                        self.logs.remove(0);
                    }
                }
                WorkerEvent::DoneUpdate => {
                    let Some(mut worker) = self.worker.clone() else {
                        return Task::none();
                    };

                    if let Some(name) = &self.operation_crate {
                        let target_crate = self.crate_list.get_mut(&name.name).unwrap();
                        target_crate.version = target_crate.crates_version.clone().unwrap();
                    }

                    return if self.delete_crates.is_empty() {
                        self.operation_crate = None;
                        self.delete_crates.clear();
                        self.update_crates.clear();
                        self.update_lerp_states_operation_container();
                        Task::none()
                    } else {
                        let crate_list = self.delete_crates.keys().cloned().collect();

                        Task::perform(
                            async move {
                                let _ = worker.send(WorkerInput::DeleteCrates(crate_list)).await;
                            },
                            |()| Message::None,
                        )
                    };
                }
                WorkerEvent::DoneDelete => {
                    if let Some(name) = &self.operation_crate {
                        self.crate_list.remove(&name.name);
                    }

                    self.delete_crates.clear();
                    self.update_crates.clear();
                    self.update_lerp_states_operation_container();
                    self.operation_crate = None;
                }
                WorkerEvent::Updating((name, index)) => {
                    if let Some(name) = &self.operation_crate {
                        let target_crate = self.crate_list.get_mut(&name.name).unwrap();
                        target_crate.version = target_crate.crates_version.clone().unwrap();
                    }

                    let operation_crate = OperationCrate {
                        name,
                        index,
                        operation_type: OperationType::Update,
                    };
                    self.operation_crate = Some(operation_crate);
                    self.update_lerp_states_operation_progress();
                }
                WorkerEvent::Deleting((name, index)) => {
                    if let Some(name) = &self.operation_crate {
                        self.crate_list.remove(&name.name);
                    }

                    let operation_crate = OperationCrate {
                        name,
                        index,
                        operation_type: OperationType::Delete,
                    };
                    self.operation_crate = Some(operation_crate);
                    self.update_lerp_states_operation_progress();
                }
                WorkerEvent::ReadyFailed => {
                    error!("Failed to start client for fetching crates info");
                }
                _ => {
                    info!("Received fetch event: {event:?}");
                }
            },
            Message::Hovering(index) => {
                self.hovering = Some(index);
            }
            Message::HoveringExit(index) => {
                if let Some(hovering) = self.hovering
                    && hovering == index
                {
                    self.hovering = None;
                }
            }
            Message::Tick => {
                self.lerp_state.lerp_all();
                self.update_lerp_states_operation_container();
                self.update_lerp_states_operation_progress();
            }
            Message::CancelOperation => {
                self.delete_crates.clear();
                self.update_crates.clear();
                self.update_lerp_states_operation_container();
            }
            Message::ApplyOperation => {
                let Some(mut worker) = self.worker.clone() else {
                    return Task::none();
                };

                let to_return;

                if !self.update_crates.is_empty() {
                    let crate_list = self.update_crates.values().cloned().collect();

                    to_return = Task::perform(
                        async move {
                            let _ = worker.send(WorkerInput::UpdateCrates(crate_list)).await;
                        },
                        |()| Message::None,
                    );
                } else if !self.delete_crates.is_empty() {
                    let crate_list = self.delete_crates.keys().cloned().collect();

                    to_return = Task::perform(
                        async move {
                            let _ = worker.send(WorkerInput::DeleteCrates(crate_list)).await;
                        },
                        |()| Message::None,
                    );
                } else {
                    to_return = Task::none();
                }

                return to_return;
            }
            Message::ShowLog => {
                self.showing = Page::Logs;
            }
            Message::ShowCrates => {
                self.showing = Page::Crates;
            }
            Message::UpdateAll => {
                for item in self.crate_list.values() {
                    if item.locked {
                        continue;
                    }

                    if let Some(crate_version) = item.crates_version.as_ref()
                        && crate_version > &item.version
                    {
                        self.update_crates.insert(item.name.clone(), item.clone());
                    }
                }
                self.update_lerp_states_operation_container();
            }
            Message::None => {}
            Message::FeatureToggle {
                crate_name,
                feature_name,
            } => {
                let target_crate = self.crate_list.get_mut(&crate_name).unwrap();

                if &feature_name == "default" {
                    target_crate.no_default_features = !target_crate.no_default_features;
                } else if target_crate.activated_features.contains(&feature_name) {
                    target_crate.activated_features.remove(&feature_name);
                } else {
                    target_crate.activated_features.insert(feature_name);
                }
            }
            Message::GitInput(event) => match event {
                GitInputEvent::HideModal => {
                    self.git_input.show_modal = false;
                }
                GitInputEvent::Submit => {
                    let target_crate = self.crate_list.get_mut(&self.git_input.crate_name).unwrap();
                    if !self.git_input.modal_text.is_empty() {
                        target_crate.git_link = Some(self.git_input.modal_text.clone());
                    }

                    self.git_input.show_modal = false;
                    self.git_input.modal_text = String::new();
                }
                GitInputEvent::Input(text) => {
                    self.git_input.modal_text = text;
                }
            },
            Message::ToggleGitLink { crate_name } => {
                let target_crate = self.crate_list.get_mut(&crate_name).unwrap();

                if target_crate.git_link.is_none() {
                    self.git_input.show_modal = true;
                    self.git_input.modal_text = String::new();
                    self.git_input.crate_name = crate_name.clone();

                    if let Some(crate_response) = &target_crate.crate_response {
                        if let Some(repo) = &crate_response.crate_data.repository {
                            self.git_input.modal_text = repo.to_string();
                        } else if let Some(repo) = &crate_response.crate_data.homepage {
                            self.git_input.modal_text = repo.to_string();
                        }
                    }
                } else {
                    target_crate.git_link = None;
                }
            }
            Message::ToggleLock(crate_name) => {
                let target_crate = self.crate_list.get_mut(&crate_name).unwrap();

                target_crate.locked = !target_crate.locked;

                self.update_crates.remove(&crate_name);
                self.delete_crates.remove(&crate_name);

                if let Some(config) = &mut self.config {
                    config.update_lock(crate_name, target_crate.locked);
                }
            }
        }

        Task::none()
    }
}
