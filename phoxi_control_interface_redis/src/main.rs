use std::error::Error;

use tokio::sync::mpsc;

use micro_sp::*;
use std::{fs::File, io::BufReader};

use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::thread;

mod core;
pub use core::interface::photoneo_control_interface;
pub use core::state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let photoneo_name = "photoneo";

    let state = state::generate_photoneo_interface_state(photoneo_name);

    // log::info!(target: &&format!("phoxi_control_interface"), "Started auto operation: '{}'.", o.name);
    log::info!(target: &&format!("phoxi_control_interface_redis"), "Spawning tasks...");

    let (tx, rx) = mpsc::channel(32);
    let state_clone = state.clone();
    tokio::task::spawn(async move { redis_state_manager(rx, state_clone).await });
    tokio::task::spawn(async move {
        photoneo_control_interface(photoneo_name, tx)
            .await
            .unwrap()
    });

    Ok(())
}
