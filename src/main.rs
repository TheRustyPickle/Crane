mod components;
mod config;
mod icon;
mod lerp;
mod message;
mod utils;
mod worker;

use crates_io_api::CrateResponse;
use dirs::home_dir;
use iced::futures::channel::mpsc::Sender;
use iced::widget::column;
use iced::{Element, Subscription, Theme, time};
use log::{LevelFilter, error, info};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::Read as _;
use std::time::Duration;

use crate::components::{OPERATION_CONTAINER, OPERATION_CONTAINER_KEY, OPERATION_PROGRESS_KEY};
use crate::config::Config;
use crate::lerp::LerpState;
use crate::message::{GitInputEvent, GitInputState, Message};
use crate::utils::{modal, parse_git_link};
use crate::worker::{WorkerInput, event_worker};

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
    worker: Option<Sender<WorkerInput>>,
    crate_list: BTreeMap<String, LocalCrate>,
    fetch_progress: Option<usize>,
    hovering: Option<usize>,
    lerp_state: LerpState,
    update_crates: HashMap<String, LocalCrate>,
    delete_crates: HashMap<String, LocalCrate>,
    operation_crate: Option<OperationCrate>,
    logs: Vec<String>,
    git_input: GitInputState,
    config: Option<Config>,
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
    pub activated_features: HashSet<String>,
    pub no_default_features: bool,
    version: Version,
    crates_version: Option<Version>,
    crate_response: Option<CrateResponse>,
    cached_features: BTreeSet<String>,
    git_link: Option<String>,
    locked: bool,
    local_hash: Option<String>,
    latest_hash: Option<String>,
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

        let config = Config::get_or_new();

        for (name, install_info) in crate_file.installs {
            let split_name = name.split(' ').collect::<Vec<&str>>();
            if split_name.len() != 3 {
                error!("Crate name {name} is not recognized. Skipping");
                continue;
            }

            let name = split_name[0];
            let version = split_name[1];

            let source = split_name[2];

            let parsed_info = parse_git_link(source);

            let Ok(version) = Version::parse(version) else {
                error!("Failed to parse version {version} for crate {name}. Skipping");
                continue;
            };

            let mut crates_version = None;
            let mut cached_features = BTreeSet::new();
            let mut description = "This crate has no description".to_string();
            let mut locked = false;

            let mut local_hash = None;
            let mut git_link = None;

            if let Some((link, hash)) = parsed_info {
                local_hash = Some(hash);
                git_link = Some(link);
            }

            if let Some(config) = &config
                && let Some(crate_info) = config.crate_cache.get(name)
            {
                if let Some(version_string) = &crate_info.crate_version {
                    let version = Version::parse(version_string).unwrap();
                    crates_version = Some(version);
                }

                cached_features = crate_info.features.clone();
                description = crate_info.description.clone();
                locked = crate_info.locked;
            }

            let local_crate = LocalCrate {
                name: name.to_string(),
                description,
                version,
                activated_features: install_info.features.into_iter().collect(),
                no_default_features: install_info.no_default_features,
                crates_version,
                crate_response: None,
                cached_features,
                git_link,
                local_hash,
                latest_hash: None,
                locked,
            };

            crate_list.insert(name.to_string(), local_crate);
        }

        info!("Loaded {} crates", crate_list.len());

        let fetch_progress = if crate_list.is_empty() { None } else { Some(0) };

        Self {
            showing: Page::Crates,
            worker: None,
            crate_list,
            fetch_progress,
            hovering: None,
            lerp_state: LerpState::new(0.3),
            update_crates: HashMap::new(),
            delete_crates: HashMap::new(),
            operation_crate: None,
            logs: Vec::new(),
            git_input: GitInputState::default(),
            config,
        }
    }

    fn title(&self) -> String {
        "Crane".to_string()
    }

    fn view(&self) -> Element<'_, Message> {
        let mut to_render = column![];
        match self.showing {
            Page::Crates => {
                to_render = to_render.push(self.crate_items());
            }
            Page::Logs => {
                to_render = to_render.push(self.log_page());
            }
        }

        if self.fetch_progress.is_some() {
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

        if self.git_input.show_modal {
            return modal(
                to_render,
                self.git_modal(),
                Message::GitInput(GitInputEvent::HideModal),
            );
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

        let currently_at = match ongoing_operation.operation_type {
            OperationType::Update => ongoing_operation.index,
            OperationType::Delete => self.update_crates.len() + ongoing_operation.index,
        };

        let progress_status = (currently_at as f64 / total_operation as f64) * 100.0;

        self.lerp_state
            .lerp(OPERATION_PROGRESS_KEY, progress_status);
    }
}
