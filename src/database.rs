pub mod connection;
pub mod single_value;
pub mod migrations;
pub mod context;
pub mod sqlite;

use context::DbContext;

pub fn init() {

    let db = DbContext::new().init();

    // db.init_level_0_packet_store();
    // db.init_single_value_store();
}