mod components;
mod icon;
mod lerp;
mod utils;
mod worker;

use dirs::home_dir;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use iced::widget::column;
use iced::{Element, Subscription, Task, Theme, time, window};
use log::{LevelFilter, error, info};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read as _;
use std::time::Duration;

use crate::components::{
    FETCH_PROGRESS_HEIGHT, FETCH_PROGRESS_HEIGHT_KEY, FETCH_PROGRESS_KEY, OPERATION_CONTAINER,
    OPERATION_CONTAINER_KEY, OPERATION_PROGRESS_KEY,
};
use crate::lerp::LerpState;
use crate::worker::{FetchEvent, FetcherInput, event_worker};

pub fn main() -> iced::Result {
    pretty_env_logger::formatted_timed_builder()
        .format_timestamp_millis()
        .filter_module(env!("CARGO_BIN_NAME"), LevelFilter::Info)
        .init();

    iced::application(MainWindow::new, MainWindow::update, MainWindow::view)
        .subscription(MainWindow::subscription)
        .font(icon::FONT)
        .title(MainWindow::title)
        .theme(Theme::CatppuccinLatte)
        .centered()
        .run()
}

pub struct MainWindow {
    showing: Page,
    worker: Option<Sender<FetcherInput>>,
    crate_list: BTreeMap<String, LocalCrate>,
    fetch_progress: usize,
    hovering: Option<usize>,
    lerp_state: LerpState,
    update_crates: BTreeMap<String, LocalCrate>,
    delete_crates: BTreeMap<String, LocalCrate>,
    operation_crate: Option<OperationCrate>,
}

pub struct OperationCrate {
    name: String,
    operation_type: OperationType,
    index: usize,
}

pub enum OperationType {
    Update,
    Delete,
}

#[derive(Clone, Default, Copy, PartialEq, Eq)]
pub enum Page {
    #[default]
    Crates,
    Logs,
}

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    UpdatePressed(String),
    DeletePressed(String),
    FetchEvent(FetchEvent),
    Hovering(usize),
    HoveringExit(usize),
    Tick,
    CancelOperation,
    ApplyOperation,
    ShowLog,
    None,
}

#[derive(Serialize, Deserialize)]
pub struct CratesFile {
    pub installs: HashMap<String, InstallInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallInfo {
    pub version_req: Option<String>,
    pub bins: Vec<String>,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub profile: Option<String>,
    pub target: Option<String>,
    pub rustc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LocalCrate {
    name: String,
    description: String,
    pub features: Vec<String>,
    pub no_default_features: bool,
    version: Version,
    crates_version: Option<Version>,
}

impl MainWindow {
    fn new() -> Self {
        let Some(mut target_dir) = home_dir() else {
            println!("Failed to get home directory. Exiting");
            std::process::exit(1);
        };

        target_dir.push(".cargo");
        target_dir.push(".crates2.json");

        if !target_dir.exists() {
            println!("No crates2.json file found. Exiting");
            std::process::exit(1);
        }
        let mut file = File::open(&target_dir).unwrap();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();

        let crate_file: CratesFile = serde_json::from_str(&file_content).unwrap();

        let mut crate_list = BTreeMap::new();

        for (name, install_info) in crate_file.installs {
            let split_name = name.split(' ').collect::<Vec<&str>>();
            if split_name.len() != 3 {
                error!("Crate name {name} is not recognized. Skipping");
                continue;
            }

            let name = split_name[0];
            let version = split_name[1];

            // TODO:: Determine whether it was installed with git flag
            let _source = split_name[2];

            let Ok(version) = Version::parse(version) else {
                error!("Failed to parse version {version} for crate {name}. Skipping");
                continue;
            };

            let local_crate = LocalCrate {
                name: name.to_string(),
                description: "A very useful Rust utility for all your command-line needs."
                    .to_string(),
                version,
                features: install_info.features,
                no_default_features: install_info.no_default_features,
                crates_version: None,
            };

            crate_list.insert(name.to_string(), local_crate);
        }

        info!("Loaded {} crates", crate_list.len());

        Self {
            showing: Page::Crates,
            worker: None,
            crate_list,
            fetch_progress: 0,
            hovering: None,
            lerp_state: LerpState::new(0.3),
            update_crates: BTreeMap::new(),
            delete_crates: BTreeMap::new(),
            operation_crate: None,
        }
    }

    fn title(&self) -> String {
        "Main Window".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Close => return window::latest().and_then(window::close),
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
                FetchEvent::Ready(mut sender) => {
                    self.worker = Some(sender.clone());
                    let crate_names = self.crate_list.keys().cloned().collect();

                    return Task::perform(
                        async move {
                            let _ = sender.send(FetcherInput::CrateList(crate_names)).await;
                        },
                        |()| Message::None,
                    );
                }
                FetchEvent::Success((details, index)) => {
                    self.fetch_progress = index + 1;

                    let mut progress_status = 0.0;
                    let total_item = self.crate_list.len();

                    if total_item != 0 && self.fetch_progress != 0 {
                        progress_status = (self.fetch_progress as f32 / total_item as f32) * 100.0;
                    }

                    self.lerp_state
                        .lerp(FETCH_PROGRESS_KEY, progress_status as f64);

                    self.lerp_state
                        .lerp(FETCH_PROGRESS_HEIGHT_KEY, FETCH_PROGRESS_HEIGHT);

                    let description = details
                        .crate_data
                        .description
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

                    target_crate.description = description;
                    target_crate.crates_version = Some(latest_version);
                }
                FetchEvent::DoneCrateCheck => {
                    self.lerp_state.lerp("fetch_progress_height", 0.0);
                }
                FetchEvent::Log(log) => {
                    info!("{log}");
                }
                FetchEvent::DoneUpdate => {
                    let Some(mut worker) = self.worker.clone() else {
                        return Task::none();
                    };

                    return if !self.delete_crates.is_empty() {
                        let crate_list = self.delete_crates.keys().cloned().collect();

                        Task::perform(
                            async move {
                                let _ = worker.send(FetcherInput::DeleteCrates(crate_list)).await;
                            },
                            |()| Message::None,
                        )
                    } else {
                        self.operation_crate = None;
                        self.delete_crates.clear();
                        self.update_crates.clear();
                        self.update_lerp_states_operation_container();
                        Task::none()
                    };
                }
                FetchEvent::DoneDelete => {
                    self.delete_crates.clear();
                    self.update_crates.clear();
                    self.update_lerp_states_operation_container();
                    self.operation_crate = None;
                }
                FetchEvent::Updating((name, index)) => {
                    let operation_crate = OperationCrate {
                        name,
                        index,
                        operation_type: OperationType::Update,
                    };
                    self.operation_crate = Some(operation_crate);
                    self.update_lerp_states_operation_progress();
                }
                FetchEvent::Deleting((name, index)) => {
                    let operation_crate = OperationCrate {
                        name,
                        index,
                        operation_type: OperationType::Delete,
                    };
                    self.operation_crate = Some(operation_crate);
                    self.update_lerp_states_operation_progress();
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
                            let _ = worker.send(FetcherInput::UpdateCrates(crate_list)).await;
                        },
                        |()| Message::None,
                    );
                } else if !self.delete_crates.is_empty() {
                    let crate_list = self.delete_crates.keys().cloned().collect();

                    to_return = Task::perform(
                        async move {
                            let _ = worker.send(FetcherInput::DeleteCrates(crate_list)).await;
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
            Message::None => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let mut to_render = column![self.crate_items()];

        if self.fetch_progress != self.crate_list.len() {
            to_render = to_render.push(self.fetch_progress());
        } else {
            let container_height = self.lerp_state.get("fetch_progress_height");

            if container_height.unwrap_or(0.0) > 0.0 {
                to_render = to_render.push(self.fetch_progress());
            }
        }

        let container_height = self.lerp_state.get("operation_container_height");

        if container_height.unwrap_or(0.0) > 0.0 {
            to_render = to_render.push(self.operation_prompt());
        }

        to_render.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(event_worker).map(Message::FetchEvent),
            if self.lerp_state.has_active_lerps() {
                time::every(Duration::from_millis(16)).map(|_| Message::Tick)
            } else {
                Subscription::none()
            },
        ])
    }

    fn update_lerp_states_operation_container(&mut self) {
        if self.update_crates.is_empty() && self.delete_crates.is_empty() {
            self.lerp_state.lerp(OPERATION_CONTAINER_KEY, 0.0);
        } else {
            self.lerp_state
                .lerp(OPERATION_CONTAINER_KEY, OPERATION_CONTAINER);
        }
    }

    fn update_lerp_states_operation_progress(&mut self) {
        let total_operation = self.update_crates.len() + self.delete_crates.len();

        let Some(ongoing_operation) = self.operation_crate.as_ref() else {
            return;
        };

        let mut currently_at;

        match ongoing_operation.operation_type {
            OperationType::Update => {
                currently_at = ongoing_operation.index;
            }
            OperationType::Delete => {
                currently_at = self.update_crates.len() + ongoing_operation.index;
            }
        }

        currently_at += 1;

        let progress_status = (currently_at as f64 / total_operation as f64) * 100.0;

        self.lerp_state
            .lerp(OPERATION_PROGRESS_KEY, progress_status);
    }
}
