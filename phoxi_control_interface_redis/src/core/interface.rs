use crate::*;
use serde_json::Value;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

use std::sync::mpsc as sync_mpsc;

use super::state::ScanRequest;

pub async fn photoneo_control_interface(
    photoneo_name: &str,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;

        let mut request_trigger = match state.get_value(&format!("{photoneo_name}_request_trigger"))
        {
            micro_sp::SPValue::Bool(value) => value,
            _ => {
                log::error!(target: &&format!(
                    "{photoneo_name}_control_interface"),
                    "Couldn't get {} from the shared state.",
                    &format!("{photoneo_name}_request_trigger")
                );
                false
            }
        };
        let mut request_state = match state.get_value(&format!("{photoneo_name}_request_state")) {
            micro_sp::SPValue::String(value) => value,
            _ => {
                log::error!(target: &&format!(
                    "{photoneo_name}_control_interface"),
                    "Couldn't get {} from the shared state.",
                    &format!("{photoneo_name}_request_state")
                );
                SPValue::Unknown(SPValueType::String).to_string()
            }
        };

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let name_identification =
                    match state.get_value(&format!("{photoneo_name}_name_identification")) {
                        micro_sp::SPValue::String(value) => value,
                        _ => {
                            log::error!(target: &&format!(
                                "{photoneo_name}_control_interface"),
                                "Couldn't get {} from the shared state.",
                                &format!("{photoneo_name}_name_identification")
                            );
                            SPValue::Unknown(SPValueType::String).to_string()
                        }
                    };

                let hardware_identification =
                    match state.get_value(&format!("{photoneo_name}_hardware_identification")) {
                        micro_sp::SPValue::String(value) => value,
                        _ => {
                            log::error!(target: &&format!(
                                "{photoneo_name}_control_interface"),
                                "Couldn't get {} from the shared state.",
                                &format!("{photoneo_name}_hardware_identification")
                            );
                            SPValue::Unknown(SPValueType::String).to_string()
                        }
                    };

                let ip_identification =
                    match state.get_value(&format!("{photoneo_name}_ip_identification")) {
                        micro_sp::SPValue::String(value) => value,
                        _ => {
                            log::error!(target: &&format!(
                                "{photoneo_name}_control_interface"),
                                "Couldn't get {} from the shared state.",
                                &format!("{photoneo_name}_ip_identification")
                            );
                            SPValue::Unknown(SPValueType::String).to_string()
                        }
                    };

                let mut phoxi_raw_info =
                    match state.get_value(&format!("{photoneo_name}_phoxi_raw_info")) {
                        micro_sp::SPValue::String(value) => value,
                        _ => {
                            log::error!(target: &&format!(
                                "{photoneo_name}_control_interface"),
                                "Couldn't get {} from the shared state.",
                                &format!("{photoneo_name}_phoxi_raw_info")
                            );
                            SPValue::Unknown(SPValueType::String).to_string()
                        }
                    };

                let command_type = match state.get_value(&format!("{photoneo_name}_command_type")) {
                    micro_sp::SPValue::String(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_command_type")
                        );
                        SPValue::Unknown(SPValueType::String).to_string()
                    }
                };

                let scene_name = match state.get_value(&format!("{photoneo_name}_scene_name")) {
                    micro_sp::SPValue::String(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_scene_name")
                        );
                        SPValue::Unknown(SPValueType::String).to_string()
                    }
                };

                let praw = match state.get_value(&format!("{photoneo_name}_praw")) {
                    micro_sp::SPValue::Bool(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_praw")
                        );
                        true
                    }
                };

                let ply = match state.get_value(&format!("{photoneo_name}_ply")) {
                    micro_sp::SPValue::Bool(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_ply")
                        );
                        false
                    }
                };

                let tif = match state.get_value(&format!("{photoneo_name}_tif")) {
                    micro_sp::SPValue::Bool(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_tif")
                        );
                        false
                    }
                };

                let praw_dir = match state.get_value(&format!("{photoneo_name}_praw_dir")) {
                    micro_sp::SPValue::String(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_praw_dir")
                        );
                        SPValue::Unknown(SPValueType::String).to_string()
                    }
                };

                let ply_dir = match state.get_value(&format!("{photoneo_name}_ply_dir")) {
                    micro_sp::SPValue::String(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_ply_dir")
                        );
                        SPValue::Unknown(SPValueType::String).to_string()
                    }
                };

                let tif_dir = match state.get_value(&format!("{photoneo_name}_tif_dir")) {
                    micro_sp::SPValue::String(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_tif_dir")
                        );
                        SPValue::Unknown(SPValueType::String).to_string()
                    }
                };

                let timeout = match state.get_value(&format!("{photoneo_name}_timeout")) {
                    micro_sp::SPValue::Int64(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_timeout")
                        );
                        5000
                    }
                };

                let settings = match state.get_value(&format!("{photoneo_name}_settings")) {
                    micro_sp::SPValue::String(value) => value,
                    _ => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Couldn't get {} from the shared state.",
                            &format!("{photoneo_name}_settings")
                        );
                        "default".to_string()
                    }
                };

                let scan_request = ScanRequest {
                    name_identification,
                    hardware_identification,
                    ip_identification,
                    command_type,
                    scene_name,
                    praw,
                    ply,
                    tif,
                    praw_dir,
                    ply_dir,
                    tif_dir,
                    timeout,
                    settings,
                };

                match call_blocking_exec(scan_request, photoneo_name) {
                    Ok(val) => {
                        log::info!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Photoneo succeeded."
                        );
                        request_state = ServiceRequestState::Succeeded.to_string();
                        phoxi_raw_info = val[0].clone();
                    }
                    Err(e) => {
                        log::error!(target: &&format!(
                            "{photoneo_name}_control_interface"),
                            "Photoneo failed."
                        );
                        request_state = ServiceRequestState::Failed.to_string();
                        phoxi_raw_info = e.to_string();
                    }
                };

                let new_state = state
                    .update(
                        &format!("{photoneo_name}_request_trigger"),
                        request_trigger.to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_name}_request_state"),
                        request_state.to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_name}_phoxi_raw_info"),
                        phoxi_raw_info.to_spvalue(),
                    );

                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(StateManagement::SetPartialState(modified_state))
                    .await?;
            }
        }
        interval.tick().await;
    }
}

fn call_blocking_exec(request: ScanRequest, photoneo_name: &str) -> Result<Vec<String>, io::Error> {
    let args = prepare_arguments(&request, photoneo_name);
    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let (tx, rx) = sync_mpsc::channel();

    thread::spawn(move || {
        let lines: Vec<String> = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "".to_string()))
            .collect();
        tx.send(lines).unwrap();
    });

    let timeout_duration = Duration::from_millis(request.timeout as u64);

    match rx.recv_timeout(timeout_duration) {
        Ok(output_lines) => Ok(output_lines),
        Err(_) => {
            child.kill()?;
            let _ = child.wait(); // Clean up the process if it's still running
            Ok(vec!["Timeout Expired".to_string()])
        }
    }
}

fn prepare_arguments(request: &ScanRequest, photoneo_name: &str) -> Vec<String> {
    let capcom = capitalize_first(&request.command_type);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let settings_path = format!(
        "{}/parameters/settings/{}.json",
        manifest_dir, request.settings
    );
    let parameters_path = format!("{}/parameters/scanners/photoneo_volvo.json", manifest_dir);

    let settings = load_json_from_file(&settings_path, photoneo_name).unwrap();
    let parameters = load_json_from_file(&parameters_path, photoneo_name).unwrap();

    let mut args_list: Vec<String> = Vec::new();

    // 0 - executable name
    args_list.push(format!(
        "{}/cpp_executables/{}/{}_Release",
        manifest_dir, capcom, capcom
    ));

    // 1 - scanner hardware identification
    args_list.push(
        parameters["hardware_identification"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 2 - scene name
    args_list.push(request.scene_name.clone());

    // 3 - save scan in .praw format
    args_list.push(bool_to_arg(request.praw));

    // 4 - save scan in .ply format
    args_list.push(bool_to_arg(request.ply));

    // 5 - save scan in .tif format
    args_list.push(bool_to_arg(request.tif));

    // 6 - capturing_settings::shutter_multiplier
    args_list.push(
        settings["capturing_settings"]["shutter_multiplier"]["value"]
            .as_i64()
            .unwrap()
            .to_string(),
    );

    // 7 - capturing_settings::scan_multiplier
    args_list.push(
        settings["capturing_settings"]["scan_multiplier"]["value"]
            .as_i64()
            .unwrap()
            .to_string(),
    );

    // 8 - capturing_settings::resolution
    if parameters["name_identification"]
        .as_str()
        .unwrap()
        .to_string()
        == "photoneo_volvo"
    {
        args_list.push(resolution_to_arg(
            &settings["capturing_settings"]["resolution"]["min"],
        ));
    } else {
        args_list.push(resolution_to_arg(
            &settings["capturing_settings"]["resolution"]["value"],
        ));
    }

    // 9 - capturing_settings::camera_only_mode
    args_list.push(bool_to_arg(
        settings["capturing_settings"]["camera_only_mode"]["value"]
            .as_bool()
            .unwrap(),
    ));

    // 10 - capturing_settings::ambient_light_suppression
    args_list.push(bool_to_arg(
        settings["capturing_settings"]["ambient_light_suppression"]["value"]
            .as_bool()
            .unwrap(),
    ));

    // 11 - capturing_settings::coding_strategy
    args_list.push(
        settings["capturing_settings"]["coding_strategy"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 12 - capturing_settings::coding_quality
    args_list.push(
        settings["capturing_settings"]["coding_quality"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 13 - capturing_settings::texture_source
    args_list.push(
        settings["capturing_settings"]["texture_source"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 14 - capturing_settings::single_pattern_exposure
    args_list.push(
        settings["capturing_settings"]["single_pattern_exposure"]["value"]
            .as_f64()
            .unwrap()
            .to_string(),
    );

    // 15 - capturing_settings::maximum_fps
    args_list.push(
        settings["capturing_settings"]["maximum_fps"]["value"]
            .as_f64()
            .unwrap()
            .to_string(),
    );

    // 16 - capturing_settings::laser_power
    args_list.push(
        settings["capturing_settings"]["laser_power"]["value"]
            .as_i64()
            .unwrap()
            .to_string(),
    );

    // 17 - capturing_settings::projection_offset_left
    args_list.push(
        settings["capturing_settings"]["projection_offset_left"]["value"]
            .as_i64()
            .unwrap()
            .to_string(),
    );

    // 18 - capturing_settings::projection_offset_right
    args_list.push(
        settings["capturing_settings"]["projection_offset_right"]["value"]
            .as_i64()
            .unwrap()
            .to_string(),
    );

    // 19 - capturing_settings::led_power
    args_list.push(
        settings["capturing_settings"]["led_power"]["value"]
            .as_i64()
            .unwrap()
            .to_string(),
    );

    // 20 - processing_settings::max_inaccuracy
    args_list.push(
        settings["processing_settings"]["max_inaccuracy"]["value"]
            .as_f64()
            .unwrap()
            .to_string(),
    );

    // 21 - processing_settings::surface_smoothness
    args_list.push(
        settings["processing_settings"]["surface_smoothness"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 22 - processing_settings::normals_estimation_radius
    args_list.push(
        settings["processing_settings"]["normals_estimation_radius"]["value"]
            .as_i64()
            .unwrap()
            .to_string(),
    );

    // 23 - processing_settings::interreflections_filter
    args_list.push(bool_to_arg(
        settings["processing_settings"]["interreflections_filter"]["value"]
            .as_bool()
            .unwrap(),
    ));

    // 24 - experimental_settings::ambient_light_suppression_compatibility_mode
    args_list.push(bool_to_arg(
        settings["experimental_settings"]["ambient_light_suppression_compatibility_mode"]["value"]
            .as_bool()
            .unwrap(),
    ));

    // 25 - experimental_settings::pattern_decomposition_reach
    args_list.push(
        settings["experimental_settings"]["pattern_decomposition_reach"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 26 - experimental_settings::signal_contrast_threshold
    args_list.push(
        settings["experimental_settings"]["signal_contrast_threshold"]["value"]
            .as_f64()
            .unwrap()
            .to_string(),
    );

    // 27 - experimental_settings::use_extended_logging
    args_list.push(bool_to_arg(
        settings["experimental_settings"]["use_extended_logging"]["value"]
            .as_bool()
            .unwrap(),
    ));

    // 28 - Where to save the praw files
    args_list.push(request.praw_dir.clone());

    // 29 - Where to save the ply files
    args_list.push(request.ply_dir.clone());

    // 30 - Where to save the tif files
    args_list.push(request.tif_dir.clone());

    args_list
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn bool_to_arg(value: bool) -> String {
    if value {
        "1".to_string()
    } else {
        "0".to_string()
    }
}

fn resolution_to_arg(value: &Value) -> String {
    // "0" is 2064x1544 and "1" is 1032x772
    if value["width"] == 2064 && value["height"] == 1544 {
        "0".to_string()
    } else if value["width"] == 1032 && value["height"] == 772 {
        "1".to_string()
    } else {
        panic!("Unsupported Photoneo resolution")
    }
}

fn load_json_from_file(path: &str, photoneo_name: &str) -> Option<Value> {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(json) => Some(json),
                Err(e) => {
                    log::warn!(target: &&format!(
                        "{photoneo_name}_control_interface"),
                        concat!(
                            "Deserialization failed with: '{}'. ",
                            "The JSON file may be malformed or contain ",
                            "unexpected data."
                        ),
                        e
                    );
                    None
                }
            }
        }
        Err(e) => {
            log::warn!(target: &&format!(
                "{photoneo_name}_control_interface"),
                concat!(
                    "Opening json file failed with: '{}'. ",
                    "Please check if the file path is correct and ",
                    "you have sufficient permissions."
                ),
                e
            );
            None
        }
    }
}
