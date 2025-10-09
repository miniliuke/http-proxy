use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {

    #[arg(short, long)]
    pub target: String,

    #[arg(short, long, default_value_t = 18081)]
    pub port: u16,
}