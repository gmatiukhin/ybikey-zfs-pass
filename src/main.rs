use std::process::Command;

use clap::Parser;

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
            use chbs::passphrase;

            println!("Passphrase: {:?}", passphrase());
        }
    }
}
