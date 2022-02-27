use anyhow::Result;
use structopt::StructOpt;
use tokio::runtime;

pub use grammers_tl_types as tl;

mod client;
mod operation;
mod utils;

use crate::{client::Client, operation::Operation, utils::init_logger};

const DEFAULT_SESSION_FILE: &str = "default.session";

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(flatten)]
    global: GlobalOptions,

    #[structopt(subcommand)]
    op: Operation,
}

#[derive(Debug, StructOpt)]
struct GlobalOptions {
    /// Verbose of logging
    #[structopt(short, long, parse(from_occurrences), global = true)]
    pub verbose: u8,

    /// Session file name. If file does not exist it will be created
    #[structopt(long, default_value = DEFAULT_SESSION_FILE)]
    pub session: String,

    #[structopt(long = "id", env = "TG_ID", global = true)]
    pub api_id: Option<i32>,

    #[structopt(long = "hash", env = "TG_HASH", global = true)]
    pub api_hash: Option<String>,
}

async fn async_main() -> Result<()> {
    let args = Args::from_args();
    init_logger(args.global.verbose)?;

    let api_id = args.global.api_id.clone().expect("API ID is not specified");
    let api_hash = args
        .global
        .api_hash
        .clone()
        .expect("API HASH is not specified");

    let client = Client::connect(api_id, &api_hash, &args.global.session).await?;

    args.op.execute(client).await?;

    Ok(())
}

fn main() -> Result<()> {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}
