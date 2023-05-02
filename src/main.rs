use sha1::{Digest, Sha1};
use std::{fs::read, process::Command};

fn main() {
    if let Ok(true) = Command::new("ykinfo")
        .args(["-q", "-s"])
        .output()
        .map(|o| o.status.success())
    {
        if let Ok(challenge) = rpassword::prompt_password("Please enter YubiKey challenge: ") {
            if let Ok(output) = Command::new("ykchalresp")
                .arg("-2")
                .arg(challenge.trim())
                .output()
            {
                if output.status.success() {
                    if let Ok(resp) = String::from_utf8(output.stdout) {
                        if let Ok(secret) = read("secret") {
                            let pass = secret
                                .into_iter()
                                .zip(hex::decode(resp.trim()).unwrap())
                                .map(|(a, b)| a ^ b)
                                .collect::<Vec<_>>();
                            println!("{pass:x?}");
                            return;
                        }
                    }
                }
            }
        }
    }

    if let Ok(passphrase) = rpassword::prompt_password("Please enter passphrase: ") {
        let mut hasher = Sha1::new();
        hasher.update(passphrase.trim().as_bytes());
        let res = hasher.finalize();
        println!("{:x?}", res);
    }
}
