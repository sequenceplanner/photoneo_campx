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
    let photoneo_name = "photoneo";
    let mut interval = interval(Duration::from_millis(100));
    let state = state::generate_photoneo_interface_state(photoneo_name);

    log::info!(target: &&format!("phoxi_control_interface"), "Starting.");

    let (tx, rx) = mpsc::channel(32);
    let state_clone = state.clone();
    tokio::task::spawn(async move { redis_state_manager(rx, state_clone).await });

    tokio::task::spawn(async move {
        photoneo_control_interface(photoneo_name.to_string(), tx)
            .await
            .expect("Error")
    });

    loop {
        interval.tick().await;
    }

    // Ok(())
}
