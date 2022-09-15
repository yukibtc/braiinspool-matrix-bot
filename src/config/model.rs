// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::path::PathBuf;

pub struct Matrix {
    pub db_path: PathBuf,
    pub state_path: PathBuf,
    pub homeserver_url: String,
    pub proxy: Option<String>,
    pub user_id: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ConfigFileMatrix {
    pub homeserver_url: String,
    pub proxy: Option<String>,
    pub user_id: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Config {
    pub main_path: PathBuf,
    pub log_level: log::Level,
    pub proxy: Option<String>,
    pub matrix: Matrix,
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub main_path: Option<PathBuf>,
    pub log_level: Option<String>,
    pub proxy: Option<String>,
    pub matrix: ConfigFileMatrix,
}

impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ db_path: {:?}, state_path: {:?}, homeserver_url: {}, proxy: {:?}, user_id: {} }}",
            self.db_path, self.state_path, self.homeserver_url, self.proxy, self.user_id
        )
    }
}
