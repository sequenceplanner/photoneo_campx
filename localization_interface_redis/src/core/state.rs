use micro_sp::*;

pub fn generate_photoneo_localization_interface_state(photoneo_name: &str) -> State {
    let state = State::new();

    let request_trigger = bv!(&&format!("{}_localization_request_trigger", photoneo_name));
    let request_state = v!(&&format!("{}_localization_request_state", photoneo_name));
    let total_fail_counter = iv!(&&format!("{}_localization_total_fail_counter", photoneo_name));
    let subsequent_fail_counter = iv!(&&format!("{}_localization_subsequent_fail_counter", photoneo_name));

    let state = state.add(assign!(request_trigger, false.to_spvalue()));
    let state = state.add(assign!(request_state, "initial".to_spvalue()));
    let state = state.add(assign!(total_fail_counter, 0.to_spvalue()));
    let state = state.add(assign!(subsequent_fail_counter, 0.to_spvalue()));

    let scene_name = v!(&&format!("{}_localization_scene_name", photoneo_name));
    let target_name = v!(&&format!("{}_localization_target_name", photoneo_name));
    let source_format = v!(&&format!("{}_localization_source_format", photoneo_name));
    let stop_at_timeout = iv!(&&format!("{}_localization_stop_at_timeout", photoneo_name));
    let stop_at_number = iv!(&&format!("{}_localization_stop_at_number", photoneo_name));
    let success = bv!(&&format!("{}_localization_success", photoneo_name));
    let stop_criteria_met = bv!(&&format!("{}_localization_stop_criteria_met", photoneo_name));
    let count = iv!(&&format!("{}_localization_count", photoneo_name));
    let transforms = av!(&&format!("{}_localization_transforms", photoneo_name));
    let settings = v!(&&format!("{}_localization_settings", photoneo_name));

    let state = state.add(assign!(scene_name, SPValue::String(StringOrUnknown::UNKNOWN)));
    let state = state.add(assign!(target_name, SPValue::String(StringOrUnknown::UNKNOWN)));
    let state = state.add(assign!(source_format, SPValue::String(StringOrUnknown::UNKNOWN)));
    let state = state.add(assign!(stop_at_timeout, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    let state = state.add(assign!(stop_at_number, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    let state = state.add(assign!(success, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    let state = state.add(assign!(stop_criteria_met, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    let state = state.add(assign!(count, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    let state = state.add(assign!(transforms, SPValue::Array(ArrayOrUnknown::UNKNOWN)));
    let state = state.add(assign!(settings, SPValue::String(StringOrUnknown::UNKNOWN)));

    state
}

pub struct LocalizeRequest {
    pub scene_name: String,  // Where to look
    pub target_name: String, // What to look for
    pub source_format: String, // praw, ply
    pub stop_at_timeout: i64, // Timeout criterion
    pub stop_at_number: i64, // Number of detected items criterion
    pub praw_dir: String,
    pub ply_dir: String,
    pub plcf_dir: String,
    pub settings: String
}