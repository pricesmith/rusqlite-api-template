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
        app_name: "TLM Server".to_string(),
        db: PathBuf::from("./tlm.db"),
        test_db: PathBuf::from("./test.db"),
    }
}

fn main() {
    match err_main() {
        Err(e) => error!("error: {:?}", e),
        Ok(_) => (),
    }
}

#[actix_web::main]
async fn err_main() -> Result<(), Box<dyn Error>> {
    // initialize tracing
    // tracing_subscriber::fmt::init();

    // initialize logger
    // env_logger::init();
    
    // define server config
    // this will also take care of initializing from cli
    let config = define_config();

    // create db
    // dotenv().ok();
    // db::create_db("DATABSE_URL");

    // start the server
    info!("Starting server...");
    server::start(config).await?;

    Ok(())
}