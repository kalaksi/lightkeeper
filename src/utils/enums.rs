use serde_derive::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum HostStatus {
    Up,
    Down,
}