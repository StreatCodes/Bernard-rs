use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    CheckSystemInfo(CheckSystemInfoRequest),
    CheckHealth(CheckHealthRequest)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    CheckSystemInfo(CheckSystemInfoResponse),
    CheckHealth(CheckHealthResponse)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckSystemInfoRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckSystemInfoResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckHealthRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckHealthResponse {}