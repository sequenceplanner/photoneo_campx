use crate::*;
use serde_json::Value;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

use std::sync::mpsc as sync_mpsc;

use super::state::ScanRequest;

pub async fn photoneo_control_interface(
    name: String,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));

    log::info!(target: &&format!("phoxi_control_interface"), "Online.");

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;

        let mut request_trigger = state.get_bool_or_default_to_false(
            &format!("{}_control_interface", name),
            &format!("{}_request_trigger", name),
        );

        let mut request_state = state.get_string_or_default_to_unknown(
            &format!("{}_control_interface", name),
            &format!("{}_request_state", name),
        );

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let name_identification = state.get_string_or_default_to_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_name_identification", name),
                );

                let hardware_identification = state.get_string_or_default_to_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_hardware_identification", name),
                );

                let ip_identification = state.get_string_or_default_to_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_ip_identification", name),
                );

                let phoxi_raw_info;

                let command_type = state.get_string_or_default_to_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_command_type", name),
                );

                let scene_name = state.get_string_or_default_to_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_scene_name", name),
                );

                let praw = match state.get_bool_or_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_praw", name),
                ) {
                    BoolOrUnknown::UNKNOWN => true,
                    BoolOrUnknown::Bool(val) => val,
                };

                let ply = state.get_bool_or_default_to_false(
                    &format!("{}_control_interface", name),
                    &format!("{}_ply", name),
                );

                let tif = state.get_bool_or_default_to_false(
                    &format!("{}_control_interface", name),
                    &format!("{}_tif", name),
                );

                let praw_dir = state.get_string_or_default_to_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_praw_dir", name),
                );

                let ply_dir = state.get_string_or_default_to_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_ply_dir", name),
                );

                let tif_dir = state.get_string_or_default_to_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_tif_dir", name),
                );

                let timeout = match state.get_int_or_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_timeout", name),
                ) {
                    IntOrUnknown::UNKNOWN => 5000,
                    IntOrUnknown::Int64(int) => int,
                };

                let settings = match state.get_string_or_unknown(
                    &format!("{}_control_interface", name),
                    &format!("{}_tif_dir", name),
                ) {
                    StringOrUnknown::UNKNOWN => "default".to_string(),
                    StringOrUnknown::String(val) => val,
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

                match call_blocking_exec(scan_request, &name) {
                    Ok(val) => {
                        log::info!(target: &&format!(
                            "{name}_control_interface"),
                            "Photoneo succeeded."
                        );
                        request_state = ServiceRequestState::Succeeded.to_string();
                        phoxi_raw_info = val[0].clone();
                    }
                    Err(e) => {
                        log::error!(target: &&format!(
                            "{name}_control_interface"),
                            "Photoneo failed."
                        );
                        request_state = ServiceRequestState::Failed.to_string();
                        phoxi_raw_info = e.to_string();
                    }
                };

                let new_state = state
                    .update(
                        &format!("{name}_request_trigger"),
                        request_trigger.to_spvalue(),
                    )
                    .update(
                        &format!("{name}_request_state"),
                        request_state.to_spvalue(),
                    )
                    .update(
                        &format!("{name}_phoxi_raw_info"),
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
