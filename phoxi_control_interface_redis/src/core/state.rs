use micro_sp::*;

pub fn generate_photoneo_interface_state(photoneo_name: &str) -> State {
    let state = State::new();

    let request_trigger = bv!(&&format!("{}_request_trigger", photoneo_name));
    let request_state = v!(&&format!("{}_request_state", photoneo_name));
    let total_fail_counter = iv!(&&format!("{}_total_fail_counter", photoneo_name));
    let subsequent_fail_counter = iv!(&&format!("{}_subsequent_fail_counter", photoneo_name));

    let state = state.add(assign!(request_trigger, false.to_spvalue()));
    let state = state.add(assign!(request_state, "initial".to_spvalue()));
    let state = state.add(assign!(total_fail_counter, 0.to_spvalue()));
    let state = state.add(assign!(subsequent_fail_counter, 0.to_spvalue()));

    let name_identification = v!(&&format!("{}_name_identification", photoneo_name));
    let hardware_identification = v!(&&format!("{}_hardware_identification", photoneo_name));
    let ip_identification = v!(&&format!("{}_ip_identification", photoneo_name));

    let state = state.add(assign!(name_identification, SPValue::Unknown(SPValueType::String)));
    let state = state.add(assign!(hardware_identification, SPValue::Unknown(SPValueType::String)));
    let state = state.add(assign!(ip_identification, SPValue::Unknown(SPValueType::String)));

    let command_type = v!(&&format!("{}_command_type", photoneo_name));
    let scene_name = v!(&&format!("{}_scene_name", photoneo_name));
    let praw = bv!(&&format!("{}_praw", photoneo_name));
    let ply = bv!(&&format!("{}_ply", photoneo_name));
    let tif = bv!(&&format!("{}_tif", photoneo_name));
    let praw_dir = v!(&&format!("{}_praw_dir", photoneo_name));
    let ply_dir = v!(&&format!("{}_ply_dir", photoneo_name));
    let tif_dir = v!(&&format!("{}_tif_dir", photoneo_name));
    let timeout = iv!(&&format!("{}_timeout", photoneo_name));
    let settings = v!(&&format!("{}_settings", photoneo_name));

    let state = state.add(assign!(command_type, SPValue::Unknown(SPValueType::String)));
    let state = state.add(assign!(scene_name, SPValue::Unknown(SPValueType::String)));
    let state = state.add(assign!(praw, SPValue::Unknown(SPValueType::Bool)));
    let state = state.add(assign!(ply, SPValue::Unknown(SPValueType::Bool)));
    let state = state.add(assign!(tif, SPValue::Unknown(SPValueType::Bool)));
    let state = state.add(assign!(praw_dir, SPValue::Unknown(SPValueType::String)));
    let state = state.add(assign!(ply_dir, SPValue::Unknown(SPValueType::String)));
    let state = state.add(assign!(tif_dir, SPValue::Unknown(SPValueType::String)));
    let state = state.add(assign!(timeout, SPValue::Unknown(SPValueType::Int64)));
    let state = state.add(assign!(settings, SPValue::Unknown(SPValueType::String)));

    state
}

pub struct ScanRequest {
    pub name_identification: String,
    pub hardware_identification: String,
    pub ip_identification: String,
    pub command_type: String,
    pub scene_name: String,
    pub praw: bool,
    pub ply: bool,
    pub tif: bool,
    pub praw_dir: String,
    pub ply_dir: String,
    pub tif_dir: String,
    pub timeout: i64,
    pub settings: String
}