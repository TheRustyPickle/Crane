use crates_io_api::{AsyncClient, CrateResponse};
use iced::futures::StreamExt;
use iced::futures::channel::mpsc::{self, Sender};
use iced::futures::join;
use iced::task::{Never, Sipper, sipper};
use log::{error, info};
use reqwest::Client;
use serde::Deserialize;
use sipper::Sender as SSender;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::LocalCrate;

#[derive(Debug, Deserialize)]
struct Commit {
    sha: String,
}

pub fn event_worker() -> impl Sipper<Never, WorkerEvent> {
    sipper(async |mut o| {
        let (sender, mut receiver) = mpsc::channel(100);
        o.send(WorkerEvent::Ready(sender)).await;

        loop {
            let Some(event) = receiver.next().await else {
                continue;
            };

            let mut output = o.clone();

            tokio::spawn(async move {
                match event {
                    WorkerInput::GetCrateVersion(list) => {
                        let app_version = env!("CARGO_PKG_VERSION");

                        let Ok(client) = AsyncClient::new(
                            &format!("Crane/{app_version} (rusty.pickle94@gmail.com)"),
                            Duration::from_secs(1),
                        ) else {
                            error!("Failed to create client");
                            output.send(WorkerEvent::ReadyFailed).await;
                            return;
                        };
                        for (index, name) in list.into_iter().enumerate() {
                            info!("Fetching crate: {name}");
                            let resp = client.get_crate(&name).await;

                            match resp {
                                Ok(details) => {
                                    let _ = output
                                        .send(WorkerEvent::SuccessCrate((Box::new(details), index)))
                                        .await;
                                }
                                Err(e) => {
                                    error!("Failed to fetch crate: {e}");

                                    let _ = output.send(WorkerEvent::ErrorCrate(index)).await;
                                }
                            }
                        }
                        output.send(WorkerEvent::DoneCrateCheck).await;
                    }
                    WorkerInput::UpdateCrates(crate_list) => {
                        for (index, item) in crate_list.into_iter().enumerate() {
                            let mut full_command = vec![
                                String::from("cargo"),
                                String::from("install"),
                                item.name.clone(),
                            ];

                            if item.no_default_features {
                                full_command.push(String::from("--no-default-features"));
                            }

                            for feature in item.activated_features {
                                full_command.push(String::from("--features"));
                                full_command.push(feature);
                            }

                            output
                                .send(WorkerEvent::Log(format!(
                                    "Executing: {}",
                                    full_command.join(" ")
                                )))
                                .await;

                            full_command.remove(0);

                            output
                                .send(WorkerEvent::Updating((item.name.clone(), index)))
                                .await;

                            let mut command = Command::new("cargo");
                            command
                                .args(full_command)
                                .stdout(std::process::Stdio::piped())
                                .stderr(std::process::Stdio::piped());

                            run_command(&item.name, command, output.clone()).await;
                        }

                        output.send(WorkerEvent::DoneUpdate).await;
                    }
                    WorkerInput::DeleteCrates(crate_list) => {
                        for (index, item) in crate_list.into_iter().enumerate() {
                            let mut full_command = vec![
                                String::from("cargo"),
                                String::from("uninstall"),
                                item.to_string(),
                            ];

                            output
                                .send(WorkerEvent::Log(format!(
                                    "Executing: {}",
                                    full_command.join(" ")
                                )))
                                .await;

                            full_command.remove(0);

                            output
                                .send(WorkerEvent::Deleting((item.clone(), index)))
                                .await;

                            let mut command = Command::new("cargo");
                            command
                                .args(full_command)
                                .stdout(std::process::Stdio::piped())
                                .stderr(std::process::Stdio::piped());

                            run_command(&item, command, output.clone()).await;
                        }

                        output.send(WorkerEvent::DoneDelete).await;
                    }
                    WorkerInput::GetGitCommit(repo_links) => {
                        let client = Client::new();

                        for (crate_name, repo_link) in repo_links {
                            let parts: Vec<&str> =
                                repo_link.trim_end_matches('/').split('/').collect();
                            if parts.len() < 2 {
                                continue;
                            }
                            let owner = parts[parts.len() - 2];
                            let repo = parts[parts.len() - 1];

                            let api_url = format!(
                                "https://api.github.com/repos/{owner}/{repo}/commits?per_page=1",
                                owner = owner,
                                repo = repo
                            );

                            let res = client
                                .get(&api_url)
                                .header("User-Agent", "reqwest") // GitHub requires a User-Agent
                                .send()
                                .await;

                            match res {
                                Ok(response) => {
                                    let commits: Vec<Commit> =
                                        response.json().await.unwrap_or_default();

                                    let Some(commit) = commits.first() else {
                                        continue;
                                    };

                                    output
                                        .send(WorkerEvent::SuccessGitCommit {
                                            crate_name,
                                            commit: commit.sha.clone(),
                                        })
                                        .await;
                                }
                                Err(e) => {
                                    error!("Failed to fetch commit: {e}");
                                }
                            };
                        }
                    }
                }
            });
        }
    })
}

async fn run_command(item_name: &str, mut command: Command, mut output: SSender<WorkerEvent>) {
    match command.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();

            let mut stdout_lines = BufReader::new(stdout).lines();
            let mut stderr_lines = BufReader::new(stderr).lines();

            let mut output_clone = output.clone();

            let stdout_task = tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_lines.next_line().await {
                    output_clone.send(WorkerEvent::Log(line)).await
                }
            });

            let mut output_clone = output.clone();

            let stderr_task = tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_lines.next_line().await {
                    output_clone.send(WorkerEvent::Log(line)).await
                }
            });

            // Wait for cargo to finish
            let status = child.wait().await;

            // Wait for both log tasks to finish
            let _ = join!(stdout_task, stderr_task);

            match status {
                Ok(status) => {
                    let msg = format!("Finished installing {} with status: {}", item_name, status);
                    output.send(WorkerEvent::Log(msg)).await;
                }
                Err(e) => {
                    output
                        .send(WorkerEvent::Log(format!(
                            "Failed to wait on cargo for {}: {e}",
                            item_name
                        )))
                        .await;
                }
            }
        }
        Err(e) => {
            output
                .send(WorkerEvent::Log(format!(
                    "Failed to spawn cargo install for {}: {e}",
                    item_name
                )))
                .await;
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    Ready(Sender<WorkerInput>),
    ReadyFailed,
    SuccessCrate((Box<CrateResponse>, usize)),
    SuccessGitCommit { crate_name: String, commit: String },
    ErrorCrate(usize),
    Updating((String, usize)),
    Deleting((String, usize)),
    DoneCrateCheck,
    DoneUpdate,
    DoneDelete,
    Log(String),
}

pub enum WorkerInput {
    GetCrateVersion(Vec<String>),
    GetGitCommit(HashMap<String, String>),
    UpdateCrates(Vec<LocalCrate>),
    DeleteCrates(Vec<String>),
}
