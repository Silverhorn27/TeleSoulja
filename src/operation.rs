use crate::{client::Client, utils::load_channel_id_or_links};
use anyhow::Result;
use std::path::PathBuf;
use structopt::{clap::ArgGroup, StructOpt};

#[derive(Debug, StructOpt)]
pub enum Operation {
    #[structopt(group = ArgGroup::with_name("from").required(true))]
    Report {
        #[structopt(short = "m", long)]
        message: Option<String>,

        #[structopt(long, group = "from")]
        channels: Vec<String>,

        #[structopt(long, group = "from")]
        file: Option<PathBuf>,
    },
}

impl Operation {
    pub async fn execute(&self, client: Client) -> Result<()> {
        match self {
            Operation::Report {
                message,
                channels,
                file,
            } => {
                let channels = if let Some(file) = file {
                    load_channel_id_or_links(&file)?
                } else {
                    channels.clone()
                };

                for channel in channels {
                    let message = message.clone().unwrap_or(random_message());
                    if client.report_channel(&channel, message.clone()).await? {
                        println!("channel: {}, reported: {}", channel, message);
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
        Ok(())
    }
}

fn random_message() -> String {
    let messages = vec!["Фейки и дезинформация о войне"];
    messages[0].clone().to_string()
}
