
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewProcessReq {
    pub program: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewProcessResp { pub pid: u64 }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineState {
    pub beat: u64,
    pub progs: Vec<(usize, String)>
}

impl EngineState {
    pub fn new() -> Self {
        EngineState {
            beat: 0,
            progs: Vec::new()
        }
    }
}
