use clap::{Parser, Subcommand};
use ureq;

const ROOT: &'static str = "http://localhost:8000/";

#[derive(Parser, Debug)]
#[clap(author, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Water {
        #[clap(value_parser)]
        water: String,
    },
    Air {
        #[clap(value_parser)]
        air: String,
    },
}

fn main() -> Result<(), ureq::Error> {
    let _args = Args::parse();

    let res = ureq::get(ROOT).call().expect("Expected this to work");

    let status = res.status();
    if status == 200 {
        println!("Status 200, success");
    } else {
        println!("Status {}, error", res.status());
    }

    Ok(())
}
