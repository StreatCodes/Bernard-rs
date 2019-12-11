use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    CheckSystemInfo(CheckSystemInfoRequest),
    CheckHealth(CheckHealthRequest),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    CheckSystemInfo(CheckSystemInfoResponse),
    CheckHealth(CheckHealthResponse),
}

pub trait RequestHandler {
    fn respond(self) -> Response;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckSystemInfoRequest {
    pub cpu: bool,
    pub disk: bool,
    pub host: bool,
    pub memory: bool,
    pub net: bool,
    pub virt: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckSystemInfoResponse {
    pub uptime: u64,
    pub cpus: Vec<Cpu>,
    pub disks: Vec<Disk>,
    pub net: Network,
    pub memory: Memory
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckHealthRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckHealthResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Disk {
    pub physical_type: String,
    pub name: String,
    pub fs_type: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cpu {
    pub name: String,
    pub usage: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Memory {
    pub totalMemory: u64,
    pub usedMemory: u64,
    pub totalSwap: u64,
    pub usedSwap: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
    pub incoming: u64,
    pub outgoing: u64,
}