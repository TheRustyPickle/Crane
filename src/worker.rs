use crates_io_api::{AsyncClient, CrateResponse};
use iced::futures::StreamExt;
use iced::futures::channel::mpsc::{self, Sender};
use iced::futures::join;
use iced::task::{Never, Sipper, sipper};
use log::{error, info};
use sipper::Sender as SSender;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::LocalCrate;

pub fn event_worker() -> impl Sipper<Never, FetchEvent> {
    sipper(async |mut output| {
        let (sender, mut receiver) = mpsc::channel(100);
        output.send(FetchEvent::Ready(sender)).await;

        loop {
            let Some(event) = receiver.next().await else {
                continue;
            };

            match event {
                FetcherInput::CrateList(list) => {
                    let app_version = env!("CARGO_PKG_VERSION");

                    let Ok(client) = AsyncClient::new(
                        &format!("Crane/{app_version} (rusty.pickle94@gmail.com)"),
                        Duration::from_secs(1),
                    ) else {
                        error!("Failed to create client");
                        output.send(FetchEvent::ReadyFailed).await;
                        continue;
                    };
                    for (index, name) in list.into_iter().enumerate() {
                        info!("Fetching crate: {name}");
                        let resp = client.get_crate(&name).await;

                        match resp {
                            Ok(details) => {
                                let _ = output
                                    .send(FetchEvent::Success((Box::new(details), index)))
                                    .await;
                            }
                            Err(e) => {
                                error!("Failed to fetch crate: {e}");
                            }
                        }
                    }
                    output.send(FetchEvent::DoneCrateCheck).await;
                }
                FetcherInput::UpdateCrates(crate_list) => {
                    for (index, item) in crate_list.into_iter().enumerate() {
                        output
                            .send(FetchEvent::Updating((item.name.clone(), index)))
                            .await;

                        let mut command = Command::new("cargo");
                        command
                            .arg("install")
                            .arg("ego")
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped());

                        run_command(&item.name, command, output.clone()).await;
                    }

                    output.send(FetchEvent::DoneUpdate).await;
                }
                FetcherInput::DeleteCrates(crate_list) => {
                    for (index, item) in crate_list.into_iter().enumerate() {
                        output
                            .send(FetchEvent::Deleting((item.clone(), index)))
                            .await;

                        let mut command = Command::new("cargo");
                        command
                            .arg("install")
                            .arg("ego")
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped());

                        run_command(&item, command, output.clone()).await;
                    }

                    output.send(FetchEvent::DoneDelete).await;
                }
            }
        }
    })
}

async fn run_command(item_name: &str, mut command: Command, mut output: SSender<FetchEvent>) {
    match command.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();

            let mut stdout_lines = BufReader::new(stdout).lines();
            let mut stderr_lines = BufReader::new(stderr).lines();

            let mut output_clone = output.clone();

            let stdout_task = tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_lines.next_line().await {
                    output_clone.send(FetchEvent::Log(line)).await
                }
            });

            let mut output_clone = output.clone();

            let stderr_task = tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_lines.next_line().await {
                    output_clone.send(FetchEvent::Log(line)).await
                }
            });

            // Wait for cargo to finish
            let status = child.wait().await;

            // Wait for both log tasks to finish
            let _ = join!(stdout_task, stderr_task);

            match status {
                Ok(status) => {
                    let msg = format!("Finished installing {} with status: {}", item_name, status);
                    output.send(FetchEvent::Log(msg)).await;
                }
                Err(e) => {
                    output
                        .send(FetchEvent::Log(format!(
                            "Failed to wait on cargo for {}: {e}",
                            item_name
                        )))
                        .await;
                }
            }
        }
        Err(e) => {
            output
                .send(FetchEvent::Log(format!(
                    "Failed to spawn cargo install for {}: {e}",
                    item_name
                )))
                .await;
        }
    }
}

#[derive(Debug, Clone)]
pub enum FetchEvent {
    Ready(Sender<FetcherInput>),
    ReadyFailed,
    Success((Box<CrateResponse>, usize)),
    Updating((String, usize)),
    Deleting((String, usize)),
    DoneCrateCheck,
    DoneUpdate,
    DoneDelete,
    Log(String),
}

pub enum FetcherInput {
    CrateList(Vec<String>),
    UpdateCrates(Vec<LocalCrate>),
    DeleteCrates(Vec<String>),
}
