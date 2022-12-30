use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::web::{Json, self};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::packet;
use crate::errors::ServerError;
use crate::handlers::helpers::respond_json;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct PacketResponse {
    pub status: String,
    pub message: String,
}

/// Handler to call packet::receive
pub async fn post_packet(
    payload: Multipart,
    session: Session,
    config: web::Data<Config>,
) -> Result<Json<PacketResponse>, ServerError> {
    packet::receive(session, config, payload).await;
    respond_json(PacketResponse {
        status: "ok".into(),
        message: "Packet received".into(),
    })
}