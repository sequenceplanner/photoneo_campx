use std::error::Error;

use tokio::{
    sync::mpsc,
    time::{interval, Duration},
};

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
    initialize_env_logger();
    let photoneo_id = match std::env::var("PHOTONEO_ID") {
        Ok(id) => format!("phoxi_{id}"),
        Err(e) => {
            log::error!(target: &&format!("phoxi_control_interface"), "Failed to read PHOTONEO_ID with error: '{}'.", e);
            log::warn!(target: &&format!("phoxi_control_interface"), "Setting default PHOTONEO_ID 1708011.");
            "phoxi_1708011".to_string()
        }
    };
    let phoxi_scans_path = match std::env::var("PHOXI_SCANS_PATH") {
        Ok(dir) => dir,
        Err(e) => {
            log::error!(target: &&format!("phoxi_1708011_control_interface"), 
                "Failed to read PHOXI_SCANS_PATH environment variable: {}", e);
            log::warn!(target: &&format!("phoxi_1708011_control_interface"), 
                "Setting PHOXI_SCANS_PATH to /root/shared_folder/scans.");
            "/root/shared_folder/scans".to_string()
        }
    };

    let phoxi_interface_path = match std::env::var("PHOXI_INTERFACE_PATH") {
        Ok(dir) => dir,
        Err(e) => {
            log::error!(target: &&format!("phoxi_control_interface"), 
                "Failed to read PHOXI_INTERFACE_PATH environment variable: {}", e);
            log::warn!(target: &&format!("phoxi_control_interface"), 
                "Setting PHOXI_INTERFACE_PATH to /usr/local/src/photoneo_campx/phoxi_control_interface_redis.");
            "/usr/local/src/photoneo_campx/phoxi_control_interface_redis".to_string()
        }
    };

    let mut interval = interval(Duration::from_millis(100));
    let state = state::generate_photoneo_interface_state(&photoneo_id);

    log::info!(target: &&format!("phoxi_control_interface"), "Starting.");

    let (tx, rx) = mpsc::channel(32);
    let state_clone = state.clone();

    tokio::task::spawn(async move {
        match redis_state_manager(rx, state_clone).await {
            Ok(()) => (),
            Err(e) => log::error!(target: &&format!("phoxi_control_interface"), "{}", e),
        }
    });

    tokio::task::spawn(async move {
        match photoneo_control_interface(&photoneo_id, &phoxi_scans_path, &phoxi_interface_path, tx)
            .await
        {
            Ok(()) => (),
            Err(e) => log::error!(target: &&format!("phoxi_control_interface"), "{}", e),
        }
    });

    loop {
        interval.tick().await;
    }

    // Ok(())
}
