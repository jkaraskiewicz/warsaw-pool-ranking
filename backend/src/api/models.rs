// Include generated protobuf code
// The file name 'api.rs' corresponds to the package name 'api' in api.proto
include!(concat!(env!("OUT_DIR"), "/api.rs"));

// Admin login models (not in protobuf)
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}