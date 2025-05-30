// use r2r::phoxi_control_msgs::srv::Scan;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let ctx = r2r::Context::create()?;
//     let mut node = r2r::Node::create(ctx, "phoxi_control_connect_test", "")?;

//     let client = node.create_client::<Scan::Service>("/phoxi_control_interface")?;
//     let waiting_for_server = node.is_available(&client)?;

//     let _handle = tokio::task::spawn_blocking(move || loop {
//         node.spin_once(std::time::Duration::from_millis(100));
//     });

//     r2r::log_warn!("phoxi_control_connect_test", "Waiting for the server...");
//     waiting_for_server.await?;
//     r2r::log_info!("phoxi_control_connect_test", "Server available.");

//     let req_msg = r2r::phoxi_control_msgs::srv::Scan::Request {
//         command: "connect".to_string(),
//         ply: true,
//         praw: true,
//         tif: true,
//         scene_name: "test_scene".to_string(),
//         settings: "default".to_string(),
//         timeout: 3000,
//         praw_dir: "".to_string(),
//         ply_dir: "".to_string(),
//         tif_dir: "".to_string(),
//     };

//     match client.request(&req_msg) {
//         Ok(future) => match future.await {
//             Ok(response) => {
//                 if response.success {
//                     r2r::log_info!("phoxi_control_connect_test", "Connected.");
//                 } else {
//                     r2r::log_info!("phoxi_control_connect_test", "Failed.");
//                 }
//             }
//             Err(e) => {
//                 r2r::log_info!("phoxi_control_connect_test", "Failed: {e}");
//             }
//         },
//         Err(e) => {
//             r2r::log_info!("phoxi_control_connect_test", "Failed: {e}");
//         }
//     }
//     Ok(())
// }

#[tokio::main]
async fn main() {}