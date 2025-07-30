use crate::*;
use serde_json::Value;
use tokio::time::{interval, Duration};

use std::sync::{mpsc as sync_mpsc, Arc};

use super::state::ScanRequest;

pub async fn photoneo_control_interface(
    photoneo_id: &str,
    phoxi_scans_path: &str,
    phoxi_interface_path: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(250));
    let log_target = &format!("phoxi_control_interface");
    log::info!(target: &log_target, "Online.");

    let keys: Vec<String> = vec![
        &format!("{}_request_trigger", photoneo_id),
        &format!("{}_request_state", photoneo_id),
        &format!("{}_name_identification", photoneo_id),
        &format!("{}_hardware_identification", photoneo_id),
        &format!("{}_ip_identification", photoneo_id),
        &format!("{}_command_type", photoneo_id),
        &format!("{}_scene_name", photoneo_id),
        &format!("{}_praw", photoneo_id),
        &format!("{}_ply", photoneo_id),
        &format!("{}_tif", photoneo_id),
        &format!("{}_timeout", photoneo_id),
        &format!("{}_settings", photoneo_id),
    ]
    .iter()
    .map(|k| k.to_string())
    .collect();

    let mut con = connection_manager.get_connection().await;
    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let state = match StateManager::get_state_for_keys(&mut con, &keys).await {
            Some(s) => s,
            None => continue,
        };

        let mut request_trigger = state
            .get_bool_or_default_to_false(&format!("{}_request_trigger", photoneo_id), &log_target);

        let mut request_state = state.get_string_or_default_to_unknown(
            &format!("{}_request_state", photoneo_id),
            &log_target,
        );

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let name_identification = state.get_string_or_default_to_unknown(
                    &format!("{}_name_identification", photoneo_id),
                    &log_target,
                );

                let hardware_identification = state.get_string_or_default_to_unknown(
                    &format!("{}_hardware_identification", photoneo_id),
                    &log_target,
                );

                let ip_identification = state.get_string_or_default_to_unknown(
                    &format!("{}_ip_identification", photoneo_id),
                    &log_target,
                );

                let phoxi_raw_info;

                let command_type = state.get_string_or_default_to_unknown(
                    &format!("{}_command_type", photoneo_id),
                    &log_target,
                );

                let scene_name = state.get_string_or_default_to_unknown(
                    &format!("{}_scene_name", photoneo_id),
                    &log_target,
                );

                let praw = match state
                    .get_bool_or_unknown(&format!("{}_praw", photoneo_id), &log_target)
                {
                    BoolOrUnknown::UNKNOWN => true,
                    BoolOrUnknown::Bool(val) => val,
                };

                let ply = state
                    .get_bool_or_default_to_false(&format!("{}_ply", photoneo_id), &log_target);

                let tif = state
                    .get_bool_or_default_to_false(&format!("{}_tif", photoneo_id), &log_target);

                let praw_dir = format!("{phoxi_scans_path}/praw");
                let ply_dir = format!("{phoxi_scans_path}/ply");
                let tif_dir = format!("{phoxi_scans_path}/tif");

                let timeout = match state
                    .get_int_or_unknown(&format!("{}_timeout", photoneo_id), &log_target)
                {
                    IntOrUnknown::UNKNOWN => 5000,
                    IntOrUnknown::Int64(int) => int,
                };

                let settings = match state
                    .get_string_or_unknown(&format!("{}_settings", photoneo_id), &log_target)
                {
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

                match call_blocking_exec(scan_request, phoxi_interface_path, &photoneo_id) {
                    Ok(val) => {
                        log::info!(target: &log_target,
                            "Photoneo request succeeded. Check {photoneo_id}_phoxi_raw_info for feedback from the scanner."
                        );
                        request_state = ServiceRequestState::Succeeded.to_string();
                        phoxi_raw_info = val[0].clone();
                    }
                    Err(e) => {
                        log::error!(target: &log_target,
                            "Photoneo failed with error: {}.", e
                        );
                        request_state = ServiceRequestState::Failed.to_string();
                        phoxi_raw_info = e.to_string();
                    }
                };

                let new_state = state
                    .update(
                        &format!("{photoneo_id}_request_trigger"),
                        request_trigger.to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_id}_request_state"),
                        request_state.to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_id}_phoxi_raw_info"),
                        phoxi_raw_info.to_spvalue(),
                    );

                let modified_state = state.get_diff_partial_state(&new_state);
                StateManager::set_state(&mut con, &modified_state).await;
            }
        }
    }
}

fn call_blocking_exec(
    request: ScanRequest,
    phoxi_interface_path: &str,
    photoneo_id: &str,
) -> Result<Vec<String>, io::Error> {
    let args = prepare_arguments(&request, phoxi_interface_path, photoneo_id);
    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .spawn()?;

    match child.stdout.take() {
        Some(stdout) => {
            let reader = BufReader::new(stdout);

            let (tx, rx) = sync_mpsc::channel();

            thread::spawn(move || {
                let lines: Vec<String> = reader
                    .lines()
                    .map(|line| line.unwrap_or_else(|_| "".to_string()))
                    .collect();
                if let Err(e) = tx.send(lines) {
                    log::warn!(
                        target: "phoxi_control_interface",
                        "Failed to send captured stdout lines to main thread (receiver dropped) with : {}",
                        e
                    );
                }
            });

            let timeout_duration = Duration::from_millis(request.timeout as u64);

            match rx.recv_timeout(timeout_duration) {
                Ok(output_lines) => Ok(output_lines),
                Err(_) => {
                    child.kill()?;
                    let _ = child.wait();
                    Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Command execution timed out.",
                    ))
                }
            }
        }
        None => {
            log::error!(target: &&format!(
                "phoxi_control_interface"),
                "Failed to capture stdout."
            );
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Failed to capture stdout.",
            ))
        }
    }
}

fn prepare_arguments(
    request: &ScanRequest,
    phoxi_interface_path: &str,
    photoneo_id: &str,
) -> Vec<String> {
    let capcom = capitalize_first(&request.command_type);

    let settings_path = format!(
        "{}/parameters/settings/{}.json",
        phoxi_interface_path, request.settings
    );
    let parameters_path = format!(
        "{}/parameters/scanners/{}.json",
        phoxi_interface_path, photoneo_id
    );

    let settings: Value = load_json_from_file(&settings_path).unwrap_or_else(|| {
        log::warn!(
            "Failed to load settings from {}. Using built-in default settings.",
            settings_path
        );
        serde_json::from_str(crate::core::DEFAULT_SETTINGS_JSON).unwrap_or_else(|parse_err| {
            log::error!(
                "CRITICAL: Failed to parse built-in default settings JSON: {}. 
                        This indicates a bug in the default JSON string. 
                        Using Value::Null as ultimate fallback.",
                parse_err
            );
            Value::Null
        })
    });

    // For parameters, you'd have similar logic or a different default/handling:
    let parameters: Value = load_json_from_file(&parameters_path).unwrap_or_else(|| {
        log::warn!(
            "Failed to load parameters from {}. Using null as default.",
            parameters_path,
        );
        Value::Null
    });

    let mut args_list: Vec<String> = Vec::new();

    // 0 - executable photoneo_id
    args_list.push(format!(
        "{}/cpp_executables/dev/{}/{}_Release",
        phoxi_interface_path, capcom, capcom
    ));

    // 1 - scanner hardware identification
    args_list.push(request.hardware_identification.to_string());

    // 2 - scene photoneo_id
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
            .unwrap_or(1)
            .to_string(),
    );

    // 7 - capturing_settings::scan_multiplier
    args_list.push(
        settings["capturing_settings"]["scan_multiplier"]["value"]
            .as_i64()
            .unwrap_or(1)
            .to_string(),
    );

    // 8 - capturing_settings::resolution
    if parameters["name_identification"]
        .as_str()
        .unwrap_or("photoneo_1708011")
        .to_string()
        == "photoneo_1708011"
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
            .unwrap_or(false),
    ));

    // 10 - capturing_settings::ambient_light_suppression
    args_list.push(bool_to_arg(
        settings["capturing_settings"]["ambient_light_suppression"]["value"]
            .as_bool()
            .unwrap_or(false),
    ));

    // 11 - capturing_settings::coding_strategy
    args_list.push(
        settings["capturing_settings"]["coding_strategy"]["value"]
            .as_str()
            .unwrap_or("Interreflections")
            .to_string(),
    );

    // 12 - capturing_settings::coding_quality
    args_list.push(
        settings["capturing_settings"]["coding_quality"]["value"]
            .as_str()
            .unwrap_or("High")
            .to_string(),
    );

    // 13 - capturing_settings::texture_source
    args_list.push(
        settings["capturing_settings"]["texture_source"]["value"]
            .as_str()
            .unwrap_or("LED")
            .to_string(),
    );

    // 14 - capturing_settings::single_pattern_exposure
    args_list.push(
        settings["capturing_settings"]["single_pattern_exposure"]["value"]
            .as_f64()
            .unwrap_or(10.24)
            .to_string(),
    );

    // 15 - capturing_settings::maximum_fps
    args_list.push(
        settings["capturing_settings"]["maximum_fps"]["value"]
            .as_f64()
            .unwrap_or(0.0)
            .to_string(),
    );

    // 16 - capturing_settings::laser_power
    args_list.push(
        settings["capturing_settings"]["laser_power"]["value"]
            .as_i64()
            .unwrap_or(4095)
            .to_string(),
    );

    // 17 - capturing_settings::projection_offset_left
    args_list.push(
        settings["capturing_settings"]["projection_offset_left"]["value"]
            .as_i64()
            .unwrap_or(0)
            .to_string(),
    );

    // 18 - capturing_settings::projection_offset_right
    args_list.push(
        settings["capturing_settings"]["projection_offset_right"]["value"]
            .as_i64()
            .unwrap_or(0)
            .to_string(),
    );

    // 19 - capturing_settings::led_power
    args_list.push(
        settings["capturing_settings"]["led_power"]["value"]
            .as_i64()
            .unwrap_or(4095)
            .to_string(),
    );

    // 20 - processing_settings::max_inaccuracy
    args_list.push(
        settings["processing_settings"]["max_inaccuracy"]["value"]
            .as_f64()
            .unwrap_or(2.0)
            .to_string(),
    );

    // 21 - processing_settings::surface_smoothness
    args_list.push(
        settings["processing_settings"]["surface_smoothness"]["value"]
            .as_str()
            .unwrap_or("Normal")
            .to_string(),
    );

    // 22 - processing_settings::normals_estimation_radius
    args_list.push(
        settings["processing_settings"]["normals_estimation_radius"]["value"]
            .as_i64()
            .unwrap_or(2)
            .to_string(),
    );

    // 23 - processing_settings::interreflections_filter
    args_list.push(bool_to_arg(
        settings["processing_settings"]["interreflections_filter"]["value"]
            .as_bool()
            .unwrap_or(false),
    ));

    // 24 - experimental_settings::ambient_light_suppression_compatibility_mode
    args_list.push(bool_to_arg(
        settings["experimental_settings"]["ambient_light_suppression_compatibility_mode"]["value"]
            .as_bool()
            .unwrap_or(false),
    ));

    // 25 - experimental_settings::pattern_decomposition_reach
    args_list.push(
        settings["experimental_settings"]["pattern_decomposition_reach"]["value"]
            .as_str()
            .unwrap_or("Local")
            .to_string(),
    );

    // 26 - experimental_settings::signal_contrast_threshold
    args_list.push(
        settings["experimental_settings"]["signal_contrast_threshold"]["value"]
            .as_f64()
            .unwrap_or(0.032)
            .to_string(),
    );

    // 27 - experimental_settings::use_extended_logging
    args_list.push(bool_to_arg(
        settings["experimental_settings"]["use_extended_logging"]["value"]
            .as_bool()
            .unwrap_or(false),
    ));

    // 28 - Where to save the praw files
    args_list.push(request.praw_dir.clone());

    // 29 - Where to save the ply files
    args_list.push(request.ply_dir.clone());

    // 30 - Where to save the tif files
    args_list.push(request.tif_dir.clone());

    // 31 - ip Identification
    args_list.push(
        parameters["ip_identification"]
            .as_str()
            .unwrap_or("192.168.1.27")
            .to_string(),
    );

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
        log::error!(target: &&format!(
            "phoxi_control_interface"),
            "Unsupported Photoneo resolution."
        );
        log::error!(target: &&format!(
            "phoxi_control_interface"),
            "Resolution defaulting to 2064x1544."
        );
        "0".to_string()
    }
}

fn load_json_from_file(path: &str) -> Option<Value> {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(json) => Some(json),
                Err(e) => {
                    log::warn!(target: &&format!(
                        "phoxi_control_interface"),
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
                "phoxi_control_interface"),
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
