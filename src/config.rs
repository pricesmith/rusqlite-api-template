use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ip:                 String,
    pub port:               u16,
    pub db:                 PathBuf,
    pub test_db:            PathBuf,
    pub app_name:           String,
}
