use std::process::Command;

use clap::Parser;
use dicepw::params::DicewareParams;

use crate::dicepw::generate::{advance, check_pw, generate_pw};

mod dicepw;

/// Simple program to generate diceware password with YubiKey challenge response
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Challenge to sent to ykchalresp
    #[arg(short, long)]
    challenge: String,
}

fn main() {
    let args = Args::parse();
    if let Ok(output) = Command::new("ykchalresp")
        .arg("-2")
        .arg(args.challenge)
        .output()
    {
        if output.status.success() {
            let output = String::from_utf8(output.stdout).unwrap();
            println!("{}", output.trim());
        }
    }

    let params = DicewareParams {
        words: dicepw::params::Range { low: 2, high: 2 },
        extras: dicepw::params::Range { low: 0, high: 0 },
    };

    let mut seed = [0; 20];
    let target_pw = generate_pw(&seed, &params);
    let mut i = 0;
    loop {
        i += 1;
        if check_pw(&seed, &params, &target_pw).is_ok() {
            println!("FOUND: {seed:?}");
        }
        if i % 100000 == 0 {
            println!("{i}");
        }
        advance(&mut seed);
    }
}
