use crates_io_api::{AsyncClient, CrateResponse};
use iced::futures::channel::mpsc::{self, Sender};
use iced::futures::{SinkExt as _, Stream, StreamExt};
use iced::stream;
use log::{error, info};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub fn fetch_crate_updates() -> impl Stream<Item = FetchEvent> {
    stream::channel(100, move |mut output: Sender<FetchEvent>| async move {
        let (sender, mut receiver) = mpsc::channel(100);
        output.send(FetchEvent::Ready(sender)).await.unwrap();

        let mut break_loop = false;

        loop {
            if let Some(event) = receiver.next().await {
                match event {
                    FetcherInput::CrateList(list) => {
                        let app_version = env!("CARGO_PKG_VERSION");

                        let Ok(client) = AsyncClient::new(
                            &format!("Crane/{app_version} (rusty.pickle94@gmail.com)"),
                            Duration::from_secs(1),
                        ) else {
                            error!("Failed to create client");
                            break;
                        };
                        for (index, name) in list.into_iter().enumerate() {
                            let start = Instant::now();

                            info!("Fetching crate: {name}");
                            let resp = client.get_crate(&name).await;

                            match resp {
                                Ok(details) => {
                                    let _ = output
                                        .send(FetchEvent::Success((Box::new(details), index)))
                                        .await;
                                }
                                Err(_) => {
                                    let _ = output.send(FetchEvent::Error).await;
                                }
                            }

                            let elapsed = start.elapsed();
                            if elapsed < Duration::from_secs(1) {
                                sleep(Duration::from_secs(1) - elapsed).await;
                            }
                        }
                        break_loop = true;
                    }
                }
            }
            if break_loop {
                output.send(FetchEvent::Done).await.unwrap();
                break;
            }
        }
        info!("Dropping everything here");
    })
}

#[derive(Debug, Clone)]
pub enum FetchEvent {
    Ready(Sender<FetcherInput>),
    Success((Box<CrateResponse>, usize)),
    Error,
    Done,
}

pub enum FetcherInput {
    CrateList(Vec<String>),
}
