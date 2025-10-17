mod components;
mod worker;

use dirs::home_dir;
use iced::futures::SinkExt as _;
use iced::{Element, Subscription, Task, window};
use log::{LevelFilter, error, info};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read as _;

use crate::worker::{FetchEvent, FetcherInput, fetch_crate_updates};

pub fn main() -> iced::Result {
    pretty_env_logger::formatted_timed_builder()
        .format_timestamp_millis()
        .filter_module(env!("CARGO_BIN_NAME"), LevelFilter::Info)
        .init();

    iced::application(MainWindow::new, MainWindow::update, MainWindow::view)
        .subscription(MainWindow::subscription)
        .title(MainWindow::title)
        .centered()
        .run()
}

#[derive(Default)]
pub struct MainWindow {
    showing: Page,
    crate_list: Vec<Crates>,
}

#[derive(Clone, Default, Copy, PartialEq, Eq)]
pub enum Page {
    #[default]
    Crates,
}

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    UpdatePressed(String),
    FetchEvent(FetchEvent),
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

#[derive(Debug)]
pub struct Crates {
    name: String,
    description: String,
    features: Vec<String>,
    no_default_features: bool,
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

        let mut crate_list = Vec::new();

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

            crate_list.push(Crates {
                name: name.to_string(),
                description: "A very useful Rust utility for all your command-line needs."
                    .to_string(),
                version,
                features: install_info.features,
                no_default_features: install_info.no_default_features,
                crates_version: None,
            });
        }

        crate_list.sort_by(|a, b| a.name.cmp(&b.name));

        info!("Loaded {} crates", crate_list.len());

        Self {
            showing: Page::Crates,
            crate_list,
        }
    }

    fn title(&self) -> String {
        "Main Window".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Close => window::latest().and_then(window::close),
            Message::UpdatePressed(crate_name) => {
                info!("'Update' pressed for crate: {crate_name}");
                Task::none()
            }
            Message::FetchEvent(event) => {
                match event {
                    FetchEvent::Ready(mut sender) => {
                        info!("Send crate list");

                        return Task::perform(
                            async move {
                                let _ = sender
                                    .send(FetcherInput::CrateList(vec![
                                        "tokio".into(),
                                        "serde".into(),
                                        "reqwest".into(),
                                    ]))
                                    .await;
                            },
                            |()| Message::None,
                        );
                    }
                    _ => {
                        info!("Received fetch event: {event:?}");
                    }
                }
                Task::none()
            }
            Message::None => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        self.crate_items()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(fetch_crate_updates).map(Message::FetchEvent)
    }
}
