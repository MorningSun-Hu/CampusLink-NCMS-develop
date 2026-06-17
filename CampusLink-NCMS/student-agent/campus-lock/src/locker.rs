use anyhow::Result;
use std::io::{self, Write};
use tracing::{info, warn};

pub async fn run(super_password: &str) -> Result<()> {
    let mut attempts = 0;
    let max_attempts = 5;

    loop {
        print!("Password: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input == super_password {
            println!("\nAccess granted. Screen unlocked.");
            return Ok(());
        }

        attempts += 1;
        warn!("Incorrect password attempt {}/{}", attempts, max_attempts);

        if attempts >= max_attempts {
            println!("\nToo many failed attempts. System will remain locked.");
            println!("Contact administrator for assistance.");

            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            attempts = 0;
        } else {
            println!("Incorrect password. {} attempts remaining.\n", max_attempts - attempts);
        }
    }
}
