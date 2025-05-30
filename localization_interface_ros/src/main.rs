use futures::{Stream, StreamExt};
use r2r::{localization_msgs::srv::Localize, ServiceRequest};
use serde_json::Value;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::{fs::File, io::BufReader};

use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ctx = r2r::Context::create()?;
    let node = r2r::Node::create(ctx, "localization_interface", "")?;
    let arc_node = Arc::new(Mutex::new(node));

    r2r::log_info!("localization_interface", "Spawning tasks...");

    let arc_node_clone: Arc<Mutex<r2r::Node>> = arc_node.clone();
    tokio::task::spawn(async move { spawn_localization_interface(arc_node_clone).await.unwrap() });

    // keep the node alive
    let arc_node_clone: Arc<Mutex<r2r::Node>> = arc_node.clone();
    let handle = std::thread::spawn(move || loop {
        arc_node_clone
            .lock()
            .unwrap()
            .spin_once(std::time::Duration::from_millis(1000));
    });

    r2r::log_info!("localization_interface", "Node started.");

    handle.join().unwrap();

    Ok(())
}

pub async fn spawn_localization_interface(
    arc_node: Arc<Mutex<r2r::Node>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = arc_node
        .lock()
        .unwrap()
        .create_service::<Localize::Service>("/localization_interface")?;

    tokio::task::spawn(async move {
        let result = localization_interface(service).await;
        match result {
            Ok(()) => r2r::log_info!("localization_interface", "Service call succeeded."),
            Err(e) => r2r::log_error!(
                "localization_interface",
                "Service call failed with: {}.",
                e
            ),
        };
    });
    Ok(())
}

async fn localization_interface(
    mut service: impl Stream<Item = ServiceRequest<Localize::Service>> + Unpin,
) -> Result<(), Box<dyn std::error::Error>> {
    r2r::log_info!("localization_interface", "Server task spawned.");

    loop {
        match service.next().await {
            Some(request) => {
                r2r::log_info!("localization_interface", "Got request.");

                let response = match call_blocking_exec(request.message.clone()) {
                    Ok(val) => {
                        r2r::log_info!("localization_interface", "Succeeded.");
                        Localize::Response {
                            req_success: todo!(),
                            any_success: todo!(),
                            nr_of_items: todo!(),
                            transforms: todo!(),
                            raw_data: todo!(),
                        }
                    }
                    Err(e) => {
                        r2r::log_info!("localization_interface", "Failed.");
                        Localize::Response {
                            req_success: todo!(),
                            any_success: todo!(),
                            nr_of_items: todo!(),
                            transforms: todo!(),
                            raw_data: todo!(),
                        }
                    }
                };

                request
                    .respond(response)
                    .expect("Could not send service response.");
            }

            None => (),
        }
    }
}

fn call_blocking_exec(request: Localize::Request) -> Result<Vec<String>, io::Error> {
    let args = prepare_arguments(&request);
    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let lines: Vec<String> = reader
            .lines()
            .map(|line| line.unwrap_or_else(|_| "".to_string()))
            .collect();
        tx.send(lines).unwrap();
    });

    let timeout_duration = Duration::from_millis(request.stop_at_timeout as u64 + 3000);

    match rx.recv_timeout(timeout_duration) {
        Ok(output_lines) => Ok(output_lines),
        Err(_) => {
            child.kill()?;
            let _ = child.wait(); // Clean up the process if it's still running
            Ok(vec!["Timeout Expired".to_string()])
        }
    }
}


struct ParsedResult {
    success: bool,
    stop_criteria_met: bool,
    count: usize,
    results: Vec<([[f64; 4]; 4], String)>,
}

impl ParsedResult {
    fn new() -> Self {
        ParsedResult {
            success: false,
            stop_criteria_met: false,
            count: 0,
            results: Vec::new(),
        }
    }
}

fn parse_result(request: &Localize::Request, data: &[Vec<u8>]) -> ParsedResult {
    let mut parsed = ParsedResult::new();
    let mut result_lines: Vec<usize> = Vec::new();

    // Find "RESULT" lines
    for (i, line) in data.iter().enumerate() {
        let split_line: Vec<&[u8]> = line.split(|&c| c == b' ').collect();
        if split_line.contains(&&b"RESULT"[..]) {
            result_lines.push(i);
        }
    }

    // If no result lines found, return default parsed result
    if result_lines.is_empty() {
        return parsed;
    }

    // Process result lines
    for &index in &result_lines {
        if index + 2 >= data.len() {
            continue;
        }

        let m1_line = &data[index];
        let m2_line = &data[index + 1];
        let m3_line = &data[index + 2];

        let m1: Vec<f64> = m1_line
            .split(|&c| c == b' ')
            .filter_map(|x| parse_float(x))
            .collect();
        let m2: Vec<f64> = m2_line
            .split(|&c| c == b' ')
            .filter_map(|x| parse_float(x))
            .collect();
        let m3: Vec<f64> = m3_line
            .split(|&c| c == b' ')
            .filter_map(|x| parse_float(x))
            .collect();

        let m4 = vec![0.0, 0.0, 0.0, 1.0];

        // Add the parsed result
        if m1.len() == 4 && m2.len() == 4 && m3.len() == 4 {
            parsed.results.push((
                [
                    [m1[0], m1[1], m1[2], m1[3]],
                    [m2[0], m2[1], m2[2], m2[3]],
                    [m3[0], m3[1], m3[2], m3[3]],
                    [m4[0], m4[1], m4[2], m4[3]],
                ],
                request.target_name.clone(),
            ));

            r2r::log_info!("DETECTED ITEMS: ", "{:?}", parsed.results);
        }
    }

    parsed.count = parsed.results.len();

    if request.stop_at_number as usize > parsed.count {
        parsed.success = false;
        parsed.stop_criteria_met = true;
    } else {
        parsed.success = true;
        parsed.stop_criteria_met = true;
    }

    parsed
}

fn parse_float(data: &[u8]) -> Option<f64> {
    if let Ok(string) = std::str::from_utf8(data) {
        if string.contains('.') {
            return string.parse::<f64>().ok();
        }
    }
    None
}

fn prepare_arguments(request: &Localize::Request) -> Vec<String> {
    let capcom = capitalize_first(&request.command);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let settings_path = format!(
        "{}/parameters/settings/{}.json",
        manifest_dir, request.settings
    );
    let parameters_path = format!("{}/parameters/scanners/photoneo_volvo.json", manifest_dir);

    let settings = load_json_from_file(&settings_path).unwrap();
    let parameters = load_json_from_file(&parameters_path).unwrap();

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

    // 3 - target name
    args_list.push(request.target_name.clone());

    // 4 - source format
    args_list.push(request.source_format.clone());

    // 5 - stop at timeout criterion
    args_list.push(request.stop_at_timeout.to_string());

    // 6 - stop at number criterion
    args_list.push(request.stop_at_number.to_string());

    // 7 - scene noise reduction
    args_list.push(bool_to_arg(
        settings["localization_settings"]["scene_noise_reduction"]["value"]
            .as_bool()
            .unwrap()
    ));

    // 8 - smart memory
    args_list.push(bool_to_arg(
        settings["localization_settings"]["smart_memory"]["value"]
            .as_bool()
            .unwrap(),
    ));

    // 9 - scene clustering level
    args_list.push(
        settings["localization_settings"]["scene_clustering_level"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 10 - scene minimal cluster size
    args_list.push(
        settings["localization_settings"]["scene_minimal_cluster_size"]["value"]
            .as_u64()
            .unwrap()
            .to_string(),
    );

    // 11 - scene maximal cluster size
    args_list.push(
        settings["localization_settings"]["scene_maximal_cluster_size"]["value"]
            .as_u64()
            .unwrap()
            .to_string(),
    );

    // 12 - matching algorithm
    args_list.push(
        settings["localization_settings"]["matching_algorithm"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 13 - model keypoints sampling
    args_list.push(
        settings["localization_settings"]["model_keypoints_sampling"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 14 - local search radius
    args_list.push(
        settings["localization_settings"]["local_search_radius"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 15 - feature fit consideration level
    args_list.push(
        settings["localization_settings"]["feature_fit_consideration_level"]["value"]
            .as_u64()
            .unwrap()
            .to_string(),
    );

    // 16 - global maximal feature fit overflow
    args_list.push(
        settings["localization_settings"]["global_maximal_feature_fit_overflow"]["value"]
            .as_u64()
            .unwrap()
            .to_string(),
    );

    // 17 - fine alignment iterations
    args_list.push(
        settings["localization_settings"]["fine_alignment_iterations"]["value"]
            .as_u64()
            .unwrap()
            .to_string(),
    );

    // 18 - fine alignment point set
    args_list.push(
        settings["localization_settings"]["fine_alignment_point_set"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 19 - fine alignment point set sampling
    args_list.push(
        settings["localization_settings"]["fine_alignment_point_set_sampling"]["value"]
            .as_str()
            .unwrap()
            .to_string(),
    );

    // 20 - projection tolerance
    args_list.push(
        settings["localization_settings"]["projection_tolerance"]["value"]
            .as_u64()
            .unwrap()
            .to_string(),
    );

    // 21 - projection hidden part tolerance
    args_list.push(
        settings["localization_settings"]["projection_hidden_part_tolerance"]["value"]
            .as_u64()
            .unwrap()
            .to_string(),
    );

    // 22 - overlap
    args_list.push(
        settings["localization_settings"]["overlap"]["value"]
            .as_u64()
            .unwrap()
            .to_string(),
    );

    // 23 - praw location
    args_list.push(request.praw_dir.clone());

    // 24 - ply location
    args_list.push(request.ply_dir.clone());

    // 25 - plcf location
    args_list.push(request.plcf_dir.clone());

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

fn load_json_from_file(path: &str) -> Option<Value> {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(json) => Some(json),
                Err(e) => {
                    r2r::log_warn!(
                        "phoxi_control_interface",
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
            r2r::log_warn!(
                "phoxi_control_interface",
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
