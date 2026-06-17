use anyhow::Result;
use std::io::{self, Write};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod locker;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "campus_lock=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: campus-lock <super_password>");
        std::process::exit(1);
    }
    let super_password = &args[1];

    info!("CampusLock starting - screen locked");

    println!("\n==============================================");
    println!("                SCREEN LOCKED                  ");
    println!("==============================================");
    println!("  This workstation has been locked by the      ");
    println!("  campus management system.                     ");
    println!("  Enter super password to unlock.               ");
    println!("==============================================\n");

    locker::run(super_password).await?;

    info!("CampusLock exiting - screen unlocked");
    Ok(())
}
