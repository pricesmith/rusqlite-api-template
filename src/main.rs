mod config;
mod util;

pub mod server;
// pub mod database;
pub mod handlers;
pub mod errors;
pub mod packet;
pub mod database;

use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use log::{error, info};

// use clap::Arg;
// use clap::ArgMatches;
use config::Config;

fn define_config() -> Config {
    Config {
        ip: "127.0.0.1".to_string(),
        port: 8000,
        mainsite: "http://localhost:8000".to_string(),
        appname: "Outpost Ground Station - TLM Server".to_string(),
        db: PathBuf::from("./outpost.db"),
        createdirs: false,
        altmainsite: [].to_vec(),
        static_path: None,
        file_tmp_path: Path::new("./temp").to_path_buf(),
        file_path: Path::new("./files").to_path_buf(),
        error_index_note: None,
    }
}

// fn load_config(filename: &str) -> Result<Config, Box<dyn Error>> {
//     info!("loading config: {}", filename);
//     let c = toml::from_str(util::load_string(filename)?.as_str())?;
//     Ok(c)
// }

// fn get_cli_matches() -> ArgMatches<'static> {
//     return clap::App::new("Outpost.space TLM Data Server")
//         .version("1.0")
//         .author("Interstitial.coop")
//         .about("Outpost.space Ground Station TLM Server")
//         .arg(
//             Arg::with_name("config")
//                 .short("c")
//                 .long("config")
//                 .value_name("FILE")
//                 .help("specify config file")
//                 .takes_value(true),
//         )
//         .arg(
//             Arg::with_name("write_config")
//                 .short("w")
//                 .long("write_config")
//                 .value_name("FILE")
//                 .help("write default config file")
//                 .takes_value(true),
//         )
//         .get_matches();
// }

fn main() {
    match err_main() {
        Err(e) => error!("error: {:?}", e),
        Ok(_) => (),
    }
}

#[actix_web::main]
async fn err_main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    let config = define_config();

    info!("Starting server...");
    server::start(config).await?;

    Ok(())
}

    // let matches = get_cli_matches();

    // // writing a config file?
    // match matches.value_of("write_config") {
    //     Some(filename) => {
    //         util::write_string(filename, toml::to_string_pretty(&define_config())?.as_str())?;
    //         info!("default config written to file: {}", filename);
    //         return Ok(());
    //     }
    //     None => (),
    // }
    // // specifying a config file?  otherwise try to load the default.
    // let mut config = match matches.value_of("config") {
    //     Some(filename) => load_config(filename)?,
    //     None => load_config("config.toml")?,
    // };

    // // TODO upgrade when stable to -- `if !std::fs::try_exists(config.file_path)?`
    // if !std::path::Path::exists(&config.file_path) {
    //     std::fs::create_dir_all(&config.file_path)?
    // }

    // normal server ops
    // info!("server init!");
    // if config.static_path == None {
    //     for (key, value) in env::vars() {
    //         if key == "ZKNOTES_STATIC_PATH" {
    //             config.static_path = PathBuf::from_str(value.as_str()).ok();
    //         }
    //     }
    // }

    // info!("config parameters:\n\n{}", toml::to_string_pretty(&config)?);

    // info!("database init!");
    // database::store::dbinit(config.db.as_path())?;

    // server::start(config).await;