use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ip:                 String,
    pub port:               u16,
    pub db:                 PathBuf,
    pub test_db:            PathBuf,
    pub mainsite:           String,
    pub appname:            String,
    pub createdirs:         bool,
    pub altmainsite:        Vec<String>,
    pub static_path:        Option<PathBuf>,
    pub file_tmp_path:      PathBuf,
    pub file_path:          PathBuf,
}
