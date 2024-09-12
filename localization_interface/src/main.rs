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

    args_list
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// # 0 - executable name
// args_list.append(
//     os.path.join(
//         self.pholoc_driver_dir, "executables", capcom, capcom + "_Release"
//     )
// )
// # 1 - scanner hardware identification
// args_list.append(self.scanner_parameters["hardware_identification"])
// # 2 - scene name
// args_list.append(request.scene_name)
// # 3 - target name
// args_list.append(request.target_name)
// # 4 - source format
// args_list.append(request.source_format)
// # 5 - stop at timeout criterion
// args_list.append(str(request.stop_at_timeout))
// # 6 - stop at number criterion
// args_list.append(str(request.stop_at_number))
// # 7 - scene noise reduction
// args_list.append(
//     self.bool_to_arg(
//         settings["localization_settings"]["scene_noise_reduction"]["value"]
//     )
// )
// # 8 - smart memory
// args_list.append(
//     self.bool_to_arg(settings["localization_settings"]["smart_memory"]["value"])
// )
// # 9 - scene clustering level
// args_list.append(
//     settings["localization_settings"]["scene_clustering_level"]["value"]
// )
// # 10 - scene minimal cluster size
// args_list.append(
//     str(
//         settings["localization_settings"]["scene_minimal_cluster_size"]["value"]
//     )
// )
// # 11 - scene maximal cluster size
// args_list.append(
//     str(
//         settings["localization_settings"]["scene_maximal_cluster_size"]["value"]
//     )
// )
// # 12 - matching algorithm
// args_list.append(
//     settings["localization_settings"]["matching_algorithm"]["value"]
// )
// # 13 - model keypoints sampling
// args_list.append(
//     settings["localization_settings"]["model_keypoints_sampling"]["value"]
// )
// # 14 - local search radius
// args_list.append(
//     settings["localization_settings"]["local_search_radius"]["value"]
// )
// # 15 - feature fit consideration level
// args_list.append(
//     str(
//         settings["localization_settings"]["feature_fit_consideration_level"][
//             "value"
//         ]
//     )
// )
// # 16 - global maximal feature fit overflow
// args_list.append(
//     str(
//         settings["localization_settings"][
//             "global_maximal_feature_fit_overflow"
//         ]["value"]
//     )
// )
// # 17 - fine alignment iterations
// args_list.append(
//     str(settings["localization_settings"]["fine_alignment_iterations"]["value"])
// )
// # 18 - fine alignment point set
// args_list.append(
//     settings["localization_settings"]["fine_alignment_point_set"]["value"]
// )
// # 19 - fine alignment point set sampling
// args_list.append(
//     settings["localization_settings"]["fine_alignment_point_set_sampling"][
//         "value"
//     ]
// )
// # 20 - projection tolerance
// args_list.append(
//     str(settings["localization_settings"]["projection_tolerance"]["value"])
// )
// # 21 - projection hidden part tolerance
// args_list.append(
//     str(
//         settings["localization_settings"]["projection_hidden_part_tolerance"][
//             "value"
//         ]
//     )
// )
// # 22 - overlap
// args_list.append(str(settings["localization_settings"]["overlap"]["value"]))
// # 23 - praws location
// args_list.append(self.praws_dir)
// # 24 - plys location
// args_list.append(self.plys_dir)
// # 25 - plcfs location
// args_list.append(self.plcfs_dir)

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
