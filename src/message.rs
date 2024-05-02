use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GreeRequest {
    pub message: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GreetResponse {
    pub message: String
}