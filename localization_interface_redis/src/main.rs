use std::{error::Error, sync::Arc};

use tokio::time::{interval, Duration};

use micro_sp::*;
use std::{fs::File, io::BufReader};

use std::io::BufRead;
use std::process::{Command, Stdio};
use std::thread;

mod core;
pub use core::interface::photoneo_localization_interface;
pub use core::state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    initialize_env_logger();
    let photoneo_id = match std::env::var("PHOTONEO_ID") {
        Ok(id) => format!("phoxi_{id}"),
        Err(e) => {
            log::error!(target: &&format!("phoxi_localization_interface"), "Failed to read PHOTONEO_ID with error: '{}'.", e);
            log::warn!(target: &&format!("phoxi_localization_interface"), "Setting default PHOTONEO_ID 1708011.");
            "photoneo_1708011".to_string()
        }
    };
    let phoxi_scans_path = match std::env::var("PHOXI_SCANS_PATH") {
        Ok(dir) => dir,
        Err(e) => {
            log::error!(target: &&format!("phoxi_localization_interface"), 
                "Failed to read PHOXI_SCANS_PATH environment variable: {}", e);
            log::warn!(target: &&format!("phoxi_localization_interface"), 
                "Setting PHOXI_SCANS_PATH to /root/shared_folder/scans.");
            "/root/shared_folder/scans".to_string()
        }
    };

    let plcfs_path = match std::env::var("PLCFS_PATH") {
        Ok(dir) => dir,
        Err(e) => {
            log::error!(target: &&format!("phoxi_localization_interface"), 
                "Failed to read PLCFS_PATH environment variable: {}", e);
            log::warn!(target: &&format!("phoxi_localization_interface"), 
                "Setting PLCFS_PATH to /root/shared_folder/plcfs.");
            "/root/shared_folder/plcfs".to_string()
        }
    };

    let localization_interface_path = match std::env::var("PHOLOC_INTERFACE_PATH") {
        Ok(dir) => dir,
        Err(e) => {
            log::error!(target: &&format!("phoxi_localization_interface"), 
                "Failed to read PHOLOC_INTERFACE_PATH environment variable: {}", e);
            log::warn!(target: &&format!("phoxi_localization_interface"), 
                "Setting PHOLOC_INTERFACE_PATH to /usr/local/src/photoneo_campx/localization_interface_redis.");
            "/usr/local/src/photoneo_campx/localization_interface_redis".to_string()
        }
    };

    let mut interval = interval(Duration::from_millis(100));
    let state = state::generate_photoneo_localization_interface_state(&photoneo_id);

    log::info!(target: &&format!("phoxi_control_interface"), "Starting.");

    let connection_manager = ConnectionManager::new().await;
    StateManager::set_state(&mut connection_manager.get_connection().await, &state).await;
    let con_arc = Arc::new(connection_manager);

    tokio::task::spawn(async move {
        match photoneo_localization_interface(
            &photoneo_id,
            &phoxi_scans_path,
            &plcfs_path,
            &localization_interface_path,
            &con_arc,
        )
        .await
        {
            Ok(()) => (),
            Err(e) => log::error!(target: &&format!("phoxi_localization_interface"), "{}", e),
        }
    });

    loop {
        interval.tick().await;
    }

    // Ok(())
}
