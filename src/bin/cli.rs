use std::time::Duration;

use bytes::Bytes;
use clap::{Parser, Subcommand};
use dadies::DEFAULT_PORT;

#[derive(Parser, Debug)]
#[command(name = "dadies-cli", version, author, about = "Dadies CLI")]

struct Cli {
    #[clap[subcommand]]
    command: Command,

    #[arg(id = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum Command {
    Ping {
        msg: Option<Bytes>,
    },
    Get {
        key: String,
    },
    Set {
        key: String,

        value: Bytes,

        #[arg(value_parser= duration_form_ms_str)]
        expires: Option<Duration>,
    },
    Publish {
        channel: String,
        message: Bytes,
    },

    Subscribe {
        channels: Vec<String>,
    },
}
