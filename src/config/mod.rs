// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::Parser;
use dirs::home_dir;
use log::Level;

pub mod model;

use model::*;

pub use model::Config;

fn default_dir() -> PathBuf {
    let home: PathBuf = home_dir().unwrap_or_else(|| {
        log::error!("Unknown home directory");
        std::process::exit(1)
    });
    home.join(".braiinspool_bot")
}

fn default_config_file() -> PathBuf {
    let mut default = default_dir().join("config");
    default.set_extension("toml");
    default
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, parse(from_os_str))]
    config_file: Option<PathBuf>,
}

impl Config {
    pub fn from_args() -> Self {
        let args: Args = Args::parse();

        let config_file_path: PathBuf = match args.config_file {
            Some(path) => path,
            None => default_config_file(),
        };

        let config_file: ConfigFile = match Self::read_config_file(&config_file_path) {
            Ok(data) => data,
            Err(error) => {
                log::error!("Impossible to read config file at {:?}", config_file_path);
                panic!("{}", error);
            }
        };

        let main_path: PathBuf = match config_file.main_path {
            Some(path) => path,
            None => default_dir(),
        };

        let log_level: Level = match config_file.log_level {
            Some(log_level) => Level::from_str(log_level.as_str()).unwrap_or(Level::Info),
            None => Level::Info,
        };

        let config = Self {
            main_path: main_path.clone(),
            log_level,
            proxy: config_file.proxy,
            matrix: Matrix {
                db_path: main_path.join("matrix/db"),
                state_path: main_path.join("matrix/state"),
                homeserver_url: config_file.matrix.homeserver_url,
                proxy: config_file.matrix.proxy,
                user_id: config_file.matrix.user_id,
                password: config_file.matrix.password,
            },
        };

        println!("{:?}", config);

        config
    }

    fn read_config_file(path: &Path) -> std::io::Result<ConfigFile> {
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }
}
