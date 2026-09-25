use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Registers {
    pub rip: usize,
    pub rsp: usize,
    pub rbp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CrashTelemetry{
    pub event_type: String,
    pub timestamp: SystemTime,
    pub pid: u32,
    pub signal: String,
    pub fault_address: usize,
    pub instruction_pointer: usize,
    pub faulting_module: Option<String>,
    pub module_base_address: Option<usize>,
    pub relative_offset_address: Option<usize>,
    pub registers: Registers,
}

impl CrashTelemetry{
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}