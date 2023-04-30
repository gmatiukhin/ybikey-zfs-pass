use sha1::{Digest, Sha1};
use std::{fs::read, process::Command};

fn main() {
    match Command::new("ykinfo")
        .args(["-q", "-s"])
        .output()
        .map(|o| o.status.success())
    {
        Ok(true) => {
            println!("Please enter challenge");
            let mut challenge = String::new();
            if std::io::stdin().read_line(&mut challenge).is_ok() {
                if let Ok(output) = Command::new("ykchalresp")
                    .arg("-2")
                    .arg(challenge.trim())
                    .output()
                {
                    if output.status.success() {
                        let resp = output.stdout;
                        if let Ok(secret) = read("secret") {
                            let pass = secret
                                .into_iter()
                                .zip(resp)
                                .map(|(a, b)| a ^ b)
                                .collect::<Vec<_>>();
                            println!("{pass:?}");
                        }
                    }
                }
            }
        }
        _ => {
            println!("No YubiKey found. Please enter the passphrase.");
            let mut passphrase = String::new();
            if std::io::stdin().read_line(&mut passphrase).is_ok() {
                let mut hasher = Sha1::new();
                hasher.update(passphrase.trim().as_bytes());
                let res = hasher.finalize();
                println!("{:x?}", res);
            }
        }
    }
}
