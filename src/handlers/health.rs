use actix_web::web::Json;
use serde::{Deserialize, Serialize};

use crate::errors::ServerError;
use crate::handlers::helpers::respond_json;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Handler to get the liveness of the service
pub async fn get_health() -> Result<Json<HealthResponse>, ServerError> {
    respond_json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[actix_main::test]
//     async fn test_get_health() {
//         let response = get_health().await.unwrap();
//         assert_eq!(response.into_inner().status, "ok".to_string());
//     }
// }