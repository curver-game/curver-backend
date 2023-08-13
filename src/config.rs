use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Config {
    /// Server address
    #[clap(short, long, default_value = "0.0.0.0")]
    pub address: String,

    /// Server port
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
}
