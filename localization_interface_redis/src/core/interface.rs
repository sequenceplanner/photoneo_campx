use crate::*;
// use micro_sp::*;
use nalgebra::{Matrix4, Quaternion, SymmetricEigen, UnitQuaternion, Vector3};
use ordered_float::OrderedFloat;
// use nanoid::nanoid;
use serde_json::Value;
use tokio::time::{interval, Duration};

use std::{
    sync::{mpsc as sync_mpsc, Arc},
    time::SystemTime,
};

use super::state::LocalizeRequest;

pub async fn photoneo_localization_interface(
    photoneo_id: &str,
    phoxi_scans_path: &str,
    plcfs_path: &str,
    localization_interface_path: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let log_target = &format!("phoxi_localization_interface");
    log::info!(target: &log_target, "Online.");

    let keys: Vec<String> = vec![
        &format!("{}_localization_request_trigger", photoneo_id),
        &format!("{}_localization_request_state", photoneo_id),
        &format!("{}_localization_scene_name", photoneo_id),
        &format!("{}_localization_target_name", photoneo_id),
        &format!("{}_localization_source_format", photoneo_id),
        &format!("{}_localization_stop_at_timeout", photoneo_id),
        &format!("{}_localization_stop_at_number", photoneo_id),
        &format!("{}_localization_settings", photoneo_id),
        &format!("{}_localization_scanning_frame", photoneo_id),
        &format!("{}_localization_success", photoneo_id),
        &format!("{}_localization_stop_criteria_met", photoneo_id),
        &format!("{}_localization_count", photoneo_id),
        &format!("{}_localization_transforms", photoneo_id),
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

        let mut request_trigger = state.get_bool_or_default_to_false(
            &format!("{}_localization_request_trigger", photoneo_id),
            &log_target,
        );

        let mut request_state = state.get_string_or_default_to_unknown(
            &format!("{}_localization_request_state", photoneo_id),
            &log_target,
        );

        if request_trigger {
            request_trigger = false;
            if request_state == ServiceRequestState::Initial.to_string() {
                let scene_name = state.get_string_or_default_to_unknown(
                    &format!("{}_localization_scene_name", photoneo_id),
                    &log_target,
                );

                let target_name = state.get_string_or_default_to_unknown(
                    &format!("{}_localization_target_name", photoneo_id),
                    &log_target,
                );

                let source_format = state.get_string_or_default_to_unknown(
                    &format!("{}_localization_source_format", photoneo_id),
                    &log_target,
                );

                let stop_at_timeout = state.get_int_or_default_to_zero(
                    &format!("{}_localization_stop_at_timeout", photoneo_id),
                    &log_target,
                );

                let stop_at_number = state.get_int_or_default_to_zero(
                    &format!("{}_localization_stop_at_number", photoneo_id),
                    &log_target,
                );

                let settings = state.get_string_or_default_to_unknown(
                    &format!("{}_localization_settings", photoneo_id),
                    &log_target,
                );

                let scanning_frame = state.get_string_or_default_to_unknown(
                    &format!("{}_localization_scanning_frame", photoneo_id),
                    &log_target,
                );

                let praw_dir = format!("{phoxi_scans_path}/praw");
                let ply_dir = format!("{phoxi_scans_path}/ply");
                let plcf_dir = format!("{plcfs_path}");

                let localize_request = LocalizeRequest {
                    scene_name,
                    target_name,
                    source_format,
                    stop_at_timeout,
                    stop_at_number,
                    praw_dir,
                    ply_dir,
                    plcf_dir,
                    settings,
                };

                let mut success = false;
                let mut stop_criteria_met = false;
                let mut count = 0;
                let mut transforms: Vec<SPTransformStamped> = vec![];

                match call_blocking_exec(
                    &localize_request,
                    localization_interface_path,
                    &photoneo_id,
                ) {
                    Ok(output_lines) => {
                        log::info!(target: &&format!(
                            "phoxi_localization_interface"),
                            "Localization request succeeded."
                        );
                        request_state = ServiceRequestState::Succeeded.to_string();
                        let result = parse_result(&localize_request, &output_lines);
                        let resulting_tfs = make_transforms(&result.results, &scanning_frame);
                        success = result.success;
                        stop_criteria_met = result.stop_criteria_met;
                        count = result.count;
                        transforms = resulting_tfs;
                    }
                    Err(e) => {
                        log::error!(target: &&format!(
                            "phoxi_localization_interface"),
                            "Photoneo failed with error: {}.", e
                        );
                        request_state = ServiceRequestState::Failed.to_string();
                    }
                };

                let new_state = state
                    .update(
                        &format!("{photoneo_id}_localization_request_trigger"),
                        request_trigger.to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_id}_localization_request_state"),
                        request_state.to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_id}_localization_success"),
                        success.to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_id}_localization_stop_criteria_met"),
                        stop_criteria_met.to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_id}_localization_count"),
                        (count as i64).to_spvalue(),
                    )
                    .update(
                        &format!("{photoneo_id}_localization_transforms"),
                        transforms.to_spvalue(),
                    );

                let modified_state = state.get_diff_partial_state(&new_state);
                StateManager::set_state(&mut con, &modified_state).await;
            }
        }
    }
}

pub fn call_blocking_exec(
    request: &LocalizeRequest,
    localization_interface_path: &str,
    photoneo_id: &str,
) -> Result<Vec<Vec<u8>>, String> {
    let args = prepare_arguments(request, localization_interface_path, photoneo_id);
    if args.is_empty() {
        return Err("No command arguments prepared.".to_string());
    }

    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        // .stderr(Stdio::null()) // Can we use this?
        .spawn()
        .map_err(|e| format!("Failed to spawn process '{}': {}", args[0], e))?;

    match child.stdout.take() {
        Some(stdout_pipe) => {
            let reader = BufReader::new(stdout_pipe);
            let (tx, rx) = sync_mpsc::channel::<Vec<Vec<u8>>>();
            thread::spawn(move || {
                let lines_as_bytes: Vec<Vec<u8>> = reader
                    .lines()
                    .map(|line_result| {
                        line_result
                            .unwrap_or_else(|e| {
                                log::warn!(
                                    target: "phoxi_localization_interface",
                                    "Error reading a line from stdout: {}", e
                                );
                                String::new()
                            })
                            .into_bytes()
                    })
                    .filter(|v_u8| !v_u8.is_empty()) // Filter out empty lines
                    .collect();

                if let Err(e) = tx.send(lines_as_bytes) {
                    log::warn!(
                        target: "phoxi_localization_interface",
                        "Failed to send captured stdout lines to main thread (receiver dropped): {}",
                        e
                    );
                }
            });

            let timeout_duration = Duration::from_millis(request.stop_at_timeout as u64 + 3000);

            match rx.recv_timeout(timeout_duration) {
                Ok(output_lines_bytes) => {
                    let _ = child
                        .wait()
                        .map_err(|e| format!("Error waiting for child process to exit: {}", e));
                    Ok(output_lines_bytes)
                }
                Err(recv_timeout_err) => {
                    log::warn!(
                        target: "phoxi_localization_interface",
                        "Command execution timed out or channel disconnected: {:?}", recv_timeout_err
                    );

                    if let Err(e) = child.kill() {
                        log::error!(
                            target: "phoxi_localization_interface",
                            "Failed to kill timed-out process: {}", e
                        );
                    }
                    let _ = child.wait();
                    Err(format!(
                        "Command execution timed out or channel issue: {:?}",
                        recv_timeout_err
                    ))
                }
            }
        }
        None => {
            log::error!(
                target: "phoxi_localization_interface",
                "Failed to capture stdout from child process."
            );
            let _ = child.wait();
            Err("Failed to capture stdout from child process.".to_string())
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

fn _quaternion_from_matrix(matrix: &Matrix4<f64>) -> UnitQuaternion<f64> {
    // Extract the elements of the 3x3 rotation submatrix
    let m00 = matrix[(0, 0)];
    let m01 = matrix[(0, 1)];
    let m02 = matrix[(0, 2)];
    let m10 = matrix[(1, 0)];
    let m11 = matrix[(1, 1)];
    let m12 = matrix[(1, 2)];
    let m20 = matrix[(2, 0)];
    let m21 = matrix[(2, 1)];
    let m22 = matrix[(2, 2)];

    // Build the symmetric matrix K, just like in the Python code.
    // We create the full symmetric matrix for clarity.
    let mut k = Matrix4::new(
        m00 - m11 - m22,
        m01 + m10,
        m02 + m20,
        m21 - m12,
        m01 + m10,
        m11 - m00 - m22,
        m12 + m21,
        m02 - m20,
        m02 + m20,
        m12 + m21,
        m22 - m00 - m11,
        m10 - m01,
        m21 - m12,
        m02 - m20,
        m10 - m01,
        m00 + m11 + m22,
    );
    k /= 3.0;

    // Quaternion is the eigenvector of K that corresponds to the largest eigenvalue.
    // nalgebra's SymmetricEigen is the equivalent of numpy.linalg.eigh.
    let eig = SymmetricEigen::new(k);

    // Find the index of the largest eigenvalue.
    let max_eigenvalue_index = eig.eigenvalues.imax();

    // Get the corresponding eigenvector (which is a column).
    let eigenvector = eig.eigenvectors.column(max_eigenvalue_index);

    // Reorder the eigenvector components to form the quaternion [w, x, y, z].
    // This matches the Python line: `q = V[[3, 0, 1, 2], numpy.argmax(w)]`
    let w = eigenvector[3];
    let x = eigenvector[0];
    let y = eigenvector[1];
    let z = eigenvector[2];

    let mut quat = Quaternion::new(w, x, y, z);

    // Ensure w is non-negative, as is convention.
    // `if q[0] < 0.0: numpy.negative(q, q)`
    if quat.w < 0.0 {
        quat = -quat;
    }

    // Return as a UnitQuaternion, which is normalized and represents a pure rotation.
    UnitQuaternion::from_quaternion(quat)
}

fn parse_result(request: &LocalizeRequest, data: &[Vec<u8>]) -> ParsedResult {
    let mut parsed = ParsedResult::new();
    let mut result_lines: Vec<usize> = Vec::new();

    // Find "RESULT" lines
    for (i, line) in data.iter().enumerate() {
        let split_line: Vec<&[u8]> = line.split(|&c| c == b' ').collect();
        if split_line.contains(&&b"RESULT"[..]) {
            result_lines.push(i);
        }
    }

    if result_lines.is_empty() {
        return parsed;
    }

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

            log::info!(target: &&format!(
                "phoxi_localization_interface"),
                "DETECTED ITEMS: {:?}", parsed.results
            );
        }
    }

    parsed.count = parsed.results.len();

    if parsed.count == 0 {
        parsed.success = false;
        parsed.stop_criteria_met = false;
    } else if request.stop_at_number as usize > parsed.count {
        parsed.success = true;
        parsed.stop_criteria_met = false;
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

// fn make_transforms(matrices: &[(MatrixDataInternal, String)], scanning_frame: &str) -> Vec<SPTransformStamped> {
//     matrices
//         .iter()
//         .enumerate()
//         .map(|(i, (matrix_array, child_name))| {
//             let matrix = Matrix4::from(*matrix_array);

//             // In a real app, you'd use a logger like `tracing::info!` or `log::info!`.
//             // println!("MATRIX: {}", matrix);

//             let rotation_part = matrix.fixed_slice::<3, 3>(0, 0).into_owned();
//             // let q = UnitQuaternion::from_rotation_matrix(&rotation_part);
//             let unit_quaternion = UnitQuaternion::from_matrix(&rotation_part);
//             let q: &Quaternion<f64> = unit_quaternion.quaternion();

//             // 2. Extract the translation vector from the last column.
//             let translation_part = matrix.column(3).xyz();

//             let child_frame_id = format!("{}_instance_{}", child_name, nanoid::nanoid!(6));

//         let main_tf = SPTransformStamped {
//             active_transform: true,
//             enable_transform: true,
//             time_stamp: SystemTime::now(),
//             parent_frame_id: scanning_frame.to_string(),
//             child_frame_id,
//             transform: SPTransform {
//                 translation: SPTranslation {
//                     x: OrderedFloat(translation_part.x / 1000.0),
//                     y: OrderedFloat(translation_part.y / 1000.0),
//                     z: OrderedFloat(translation_part.z / 1000.0),
//                 },
//                 rotation: SPRotation {
//                     x: OrderedFloat(q.i),
//                     y: OrderedFloat(q.j),
//                     z: OrderedFloat(q.k),
//                     w: OrderedFloat(q.w),
//                 },
//             },
//             metadata: MapOrUnknown::UNKNOWN,
//         };
//         main_tf}).collect()

//             // 3. Construct the TransformStamped message (same as before).
//         //     let mut main_tf = TransformStamped::default();
//         //     main_tf.header.frame_id = "photoneo_sensor".to_string();
//         //     main_tf.child_frame_id = format!("{}_instance_{:02}", child_name, i);
//         //     main_tf.transform.translation.x = translation_part.x / 1000.0;
//         //     main_tf.transform.translation.y = translation_part.y / 1000.0;
//         //     main_tf.transform.translation.z = translation_part.z / 1000.0;
//         //     main_tf.transform.rotation.x = quaternion.x;
//         //     main_tf.transform.rotation.y = quaternion.y;
//         //     main_tf.transform.rotation.z = quaternion.z;
//         //     main_tf.transform.rotation.w = quaternion.w;

//         //     main_tf
//         // })
//         // .collect()
// }

type MatrixDataInternal = [[f64; 4]; 4];

pub fn make_transforms(
    matrices: &[(MatrixDataInternal, String)],
    scanning_frame: &str,
) -> Vec<SPTransformStamped> {
    let mut transforms: Vec<SPTransformStamped> = Vec::new();

    for (_i, (matrix_array, name)) in matrices.iter().enumerate() {
        let m = matrix_array;
        // let transformation_matrix = Matrix4::new(
        //     m[0][0], m[1][0], m[2][0], m[3][0], m[0][1], m[1][1], m[2][1], m[3][1], m[0][2],
        //     m[1][2], m[2][2], m[3][2], m[0][3], m[1][3], m[2][3], m[3][3],
        // );
        let transformation_matrix = Matrix4::new(
            m[0][0], m[0][1], m[0][2], m[0][3], m[1][0], m[1][1], m[1][2], m[1][3], m[2][0],
            m[2][1], m[2][2], m[2][3], m[3][0], m[3][1], m[3][2], m[3][3],
        );
        // let transformation_matrix = Matrix4::from(*matrix_array);

        log::info!(target: "phoxi_localization_interface", "Processing matrix for {}: {:?}", name, transformation_matrix);

        let translation_vec = Vector3::new(m[0][3], m[1][3], m[2][3]); //transformation_matrix.column(3).xyz(); //Vector3::new(m[0][3], m[1][3], m[2][3]);

        let rotation_matrix = transformation_matrix.fixed_view::<3, 3>(0, 0).into_owned();
        // let q = quaternion_from_matrix(&transformation_matrix);

        let unit_quaternion = UnitQuaternion::from_matrix(&rotation_matrix);
        let q: &Quaternion<f64> = unit_quaternion.quaternion();

        let child_frame_id = format!("{}_instance_{}", name, nanoid::nanoid!(6));

        let main_tf = SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: scanning_frame.to_string(),
            child_frame_id,
            transform: SPTransform {
                translation: SPTranslation {
                    x: OrderedFloat(translation_vec.x / 1000.0),
                    y: OrderedFloat(translation_vec.y / 1000.0),
                    z: OrderedFloat(translation_vec.z / 1000.0),
                },
                rotation: SPRotation {
                    x: OrderedFloat(q.w),
                    y: OrderedFloat(q.i),
                    z: OrderedFloat(q.j),
                    w: OrderedFloat(q.k),
                },
            },
            metadata: MapOrUnknown::UNKNOWN,
        };

        transforms.push(main_tf);
    }

    transforms
}

fn prepare_arguments(
    request: &LocalizeRequest,
    localization_interface_path: &str,
    photoneo_id: &str,
) -> Vec<String> {
    let settings_path = format!(
        "{}/parameters/settings/{}.json",
        localization_interface_path, request.settings
    );

    let parameters_path = format!(
        "{}/parameters/scanners/{}.json",
        localization_interface_path, photoneo_id
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
        "{}/cpp_executables/dev/Localize/bin/Localize_Release",
        localization_interface_path,
    ));

    // 1 - scanner hardware identification
    args_list.push(
        parameters["hardware_identification"]
            .as_str()
            .unwrap_or("photoneo_1708011")
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
            .unwrap_or(true),
    ));

    // 8 - smart memory
    args_list.push(bool_to_arg(
        settings["localization_settings"]["smart_memory"]["value"]
            .as_bool()
            .unwrap_or(false),
    ));

    // 9 - scene clustering level
    args_list.push(
        settings["localization_settings"]["scene_clustering_level"]["value"]
            .as_str()
            .unwrap_or("Normal")
            .to_string(),
    );

    // 10 - scene minimal cluster size
    args_list.push(
        settings["localization_settings"]["scene_minimal_cluster_size"]["value"]
            .as_u64()
            .unwrap_or(200)
            .to_string(),
    );

    // 11 - scene maximal cluster size
    args_list.push(
        settings["localization_settings"]["scene_maximal_cluster_size"]["value"]
            .as_u64()
            .unwrap_or(350000)
            .to_string(),
    );

    // 12 - matching algorithm
    args_list.push(
        settings["localization_settings"]["matching_algorithm"]["value"]
            .as_str()
            .unwrap_or("Surfaces")
            .to_string(),
    );

    // 13 - model keypoints sampling
    args_list.push(
        settings["localization_settings"]["model_keypoints_sampling"]["value"]
            .as_str()
            .unwrap_or("Medium")
            .to_string(),
    );

    // 14 - local search radius
    args_list.push(
        settings["localization_settings"]["local_search_radius"]["value"]
            .as_str()
            .unwrap_or("Normal")
            .to_string(),
    );

    // 15 - feature fit consideration level
    args_list.push(
        settings["localization_settings"]["feature_fit_consideration_level"]["value"]
            .as_u64()
            .unwrap_or(15)
            .to_string(),
    );

    // 16 - global maximal feature fit overflow
    args_list.push(
        settings["localization_settings"]["global_maximal_feature_fit_overflow"]["value"]
            .as_u64()
            .unwrap_or(20)
            .to_string(),
    );

    // 17 - fine alignment iterations
    args_list.push(
        settings["localization_settings"]["fine_alignment_iterations"]["value"]
            .as_u64()
            .unwrap_or(30)
            .to_string(),
    );

    // 18 - fine alignment point set
    args_list.push(
        settings["localization_settings"]["fine_alignment_point_set"]["value"]
            .as_str()
            .unwrap_or("Surface")
            .to_string(),
    );

    // 19 - fine alignment point set sampling
    args_list.push(
        settings["localization_settings"]["fine_alignment_point_set_sampling"]["value"]
            .as_str()
            .unwrap_or("Sampled")
            .to_string(),
    );

    // 20 - projection tolerance
    args_list.push(
        settings["localization_settings"]["projection_tolerance"]["value"]
            .as_u64()
            .unwrap_or(100)
            .to_string(),
    );

    // 21 - projection hidden part tolerance
    args_list.push(
        settings["localization_settings"]["projection_hidden_part_tolerance"]["value"]
            .as_u64()
            .unwrap_or(100)
            .to_string(),
    );

    // 22 - overlap
    args_list.push(
        settings["localization_settings"]["overlap"]["value"]
            .as_f64()
            .unwrap_or(15.0)
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

// fn capitalize_first(s: &str) -> String {
//     let mut chars = s.chars();
//     match chars.next() {
//         None => String::new(),
//         Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
//     }
// }

fn bool_to_arg(value: bool) -> String {
    if value {
        "1".to_string()
    } else {
        "0".to_string()
    }
}

// fn resolution_to_arg(value: &Value) -> String {
//     // "0" is 2064x1544 and "1" is 1032x772
//     if value["width"] == 2064 && value["height"] == 1544 {
//         "0".to_string()
//     } else if value["width"] == 1032 && value["height"] == 772 {
//         "1".to_string()
//     } else {
//         log::error!(target: &&format!(
//             "phoxi_localization_interface"),
//             "Unsupported Photoneo resolution."
//         );
//         log::error!(target: &&format!(
//             "phoxi_localization_interface"),
//             "Resolution defaulting to 2064x1544."
//         );
//         "0".to_string()
//     }
// }

fn load_json_from_file(path: &str) -> Option<Value> {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(json) => Some(json),
                Err(e) => {
                    log::warn!(target: &&format!(
                        "phoxi_localization_interface"),
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
                "phoxi_localization_interface"),
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
