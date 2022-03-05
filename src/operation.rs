use crate::{client::Client, utils::load_channel_id_or_links};
use anyhow::Result;
use rand::Rng;
use std::path::PathBuf;
use structopt::{clap::ArgGroup, StructOpt};

#[derive(Debug, StructOpt)]
pub enum Operation {
    #[structopt(group = ArgGroup::with_name("from").required(true))]
    Report {
        /// Custom message for reporting
        #[structopt(short = "m", long)]
        message: Option<String>,

        /// List of channels
        #[structopt(long, group = "from")]
        channels: Vec<String>,

        /// File path to channels
        #[structopt(long, group = "from")]
        file: Option<PathBuf>,

        /// Timeout after each report
        #[structopt(long, default_value = "10")]
        timeout: u64,
    },
}

impl Operation {
    pub async fn execute(&self, client: Client) -> Result<()> {
        match self {
            Operation::Report {
                message,
                channels,
                file,
                timeout,
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
                    tokio::time::sleep(std::time::Duration::from_secs(*timeout)).await;
                }
            }
        }
        Ok(())
    }
}

fn random_message() -> String {
    let messages = vec![
        "Содержит российскую пропаганду",
        "Военная пропаганда",
        "Пропаганда насилия",
        "Фейки и дезинформация о войне",
        "Дезинформация окупантов",
        "Расжигание ненависти",
        "Расжигание вражды",
        "Разжигание межнациональной розни",
        "Российсикие фейки",
        "Diversionary activity of Russian terrorism in Ukraine",
        "Russian occupants channel",
        "Fakes and dissinformation",
        "Content againts human rights",
        "СМИ подконтрольные окупантам",
        "Пророссийские и антизападные СМИ",
        "Антизападные СМИ",
        "Распостранение дезинформации",
    ];

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..messages.len());

    messages[idx].clone().to_string()
}
