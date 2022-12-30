// mod interfaces;
// mod search;
// mod util;
// mod database;

use crate::{util, database};

use super::config::Config;
// use super::database;

use std::path::Path;
use actix_multipart::{Multipart, MultipartError};
use actix_session::Session;
use actix_web::{web, HttpResponse, Result};
use futures_util::TryStreamExt;
use rusqlite::DatabaseName;
use serde::{Serialize, Deserialize};
use serde_json;
use uuid::Uuid;


/// Receives a Multipart payload and saves it to the database.
pub async fn receive(session: Session, config: web::Data<Config>, payload: Multipart) -> HttpResponse {

    // Save the uploaded packet to the database
    let result = save(session, config, payload).await;

    // Return a JSON response with the result of the save operation
    match result {
        Ok(success_value) => HttpResponse::Ok().json(success_value),
        Err(error) => HttpResponse::InternalServerError().body(format!("{:?}", error)),
    }
}

async fn save(session: Session, config: web::Data<Config>, mut payload: Multipart) -> Result<(), SaveError> {
    let conn = database::connection::open(config.db.as_path())?;

    let (metadata, packet) = extract_files(payload).await?;
    let uuid = Uuid::new_v4().to_string();
    let now = util::now()?;

    // write to sqlite.
    let (insert_stmt, params) = create_insert_stmt(&packet);
    conn.execute(insert_stmt.as_str(), &params)?;

    // Get the row id off the BLOB we just inserted.
    let rowid = conn.last_insert_rowid();
    // Open the BLOB we just inserted for IO.
    let mut blob = conn.blob_open(DatabaseName::Main, "level0blobs", "packet", rowid, false)?;

    blob.write_at(&packet);

    Ok(())
}

/// Extracts the metadata and packet parts from the multipart payload.
async fn extract_files(mut payload: Multipart) -> Result<(Metadata, Vec<u8>), ExtractError> {

    // Initialize two vectors to store the metadata and packet data
    let mut metadata_vec = Vec::new();
    let mut packet_vec = Vec::new();

    // Iterate over each field in the multipart payload
    while let Some(mut field) = payload.try_next().await? {

        // Get the `ContentDisposition` header of the current field
        let content_disposition = field.content_disposition().unwrap();

        // Get the name of the field, or return an error if it is not found
        let filename = content_disposition.get_name().unwrap_or("Fieldname not found");

        // If the field is the `metadata` or `packet` field, add it to the corresponding vector
        if filename == "metadata" {
            while let Some(chunk) = field.try_next().await? {
                metadata_vec.extend(chunk);
            }
        } else if filename == "packet" {
            while let Some(chunk) = field.try_next().await? {
                packet_vec.extend(chunk);
            }
        }
    }

    // If metadata or packet vectors are empty, nothing was found. Return the corresponding error
    if metadata_vec.is_empty() {
        return Err(ExtractError::MissingMetadata);
    }
    if packet_vec.is_empty() {
        return Err(ExtractError::MissingPacket);
    }

    // Convert the metadata vector to a string
    let metadata_str = String::from_utf8(metadata_vec)?;
    // Parse the metadata string as a `Metadata` struct
    let metadata: Metadata  = serde_json::from_str(&metadata_str)?;
    // Convert the packet vector to a `Vec<u8>` -> (into itself?...)
    let packet = packet_vec.into();

    // Return an Ok with a tuple containing the metadata and packet
    Ok((metadata, packet))
}

fn create_insert_stmt(packet: &[u8]) -> String {
    format!("insert into level0blobs (uuid, createdate, metadata, packet) values (?, ?, ?, ZEROBLOB({}))", packet.len())
}

#[derive(Deserialize, Serialize, Debug)]
struct Metadata {
    filename: String,
    filetype: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Packet {
    uuid: String,
    createdate: String,
    metadata: Metadata,
    packet: Vec<u8>,
}

#[derive(Debug)]
enum ExtractError {
    Utf8Error(std::str::Utf8Error),
    FromUtf8Error(std::string::FromUtf8Error),
    MultipartError(MultipartError),
    // BlockingError(BlockingError<T>),
    JsonError(serde_json::Error),
    MissingMetadata,
    MissingPacket,
}

impl From<std::str::Utf8Error> for ExtractError {
    fn from(error: std::str::Utf8Error) -> Self {
        ExtractError::Utf8Error(error)
    }
}

impl From<std::string::FromUtf8Error> for ExtractError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        ExtractError::FromUtf8Error(error)
    }
}

impl From<serde_json::Error> for ExtractError {
    fn from(error: serde_json::Error) -> Self {
        ExtractError::JsonError(error)
    }
}

impl From<MultipartError> for ExtractError {
    fn from(error: MultipartError) -> Self {
        ExtractError::MultipartError(error)
    }
}

#[derive(Debug)]
enum SaveError {
    DbError(rusqlite::Error),
    ExtractError(ExtractError),
    // UtilError(UtilError),
}

impl From<rusqlite::Error> for SaveError {
    fn from(error: rusqlite::Error) -> Self {
        SaveError::DbError(error)
    }
}

impl From<ExtractError> for SaveError {
    fn from(error: ExtractError) -> Self {
        SaveError::ExtractError(error)
    }
}

// impl From<UtilError> for SaveError {
//     fn from(error: UtilError) -> Self {
//         SaveError::UtilError(error)
//     }
// }

async fn receive_file_old(session: Session, config: web::Data<Config>, mut payload: Multipart) -> HttpResponse {
    match save_file_too(session, config, payload).await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => return HttpResponse::InternalServerError().body(format!("{:?}", e)),
    }
}

async fn save_file_too(session: Session, config: web::Data<Config>, payload: Multipart) -> Result<(), SaveError> {
    let conn = database::open_connection(config.db.as_path())?;
    let temp_path = config.file_tmp_path.clone();

    match save_file(&temp_path, payload).await {
        Ok((name, file_pathp)) => Ok(()),
        Err(error) => Err(SaveError::UtilError(error)),
    }
}

async fn save_file(to_dir: &Path, mut payload: Multipart) -> Result<(String, String), ExtractError> {
    // iterate over multipart stream
    // ONLY SAVING THE FIRST FILE
    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field
            .content_disposition()
            .ok_or(simple_error::SimpleError::new("bad"))?;

        let filename = content_disposition
            .get_filename()
            .unwrap_or("filename not found");

        let wkfilename = Uuid::new_v4().to_string();

        let mut filepath = to_dir.to_path_buf();
        filepath.push(wkfilename);

        let rf = filepath.clone();

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath)).await?;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.try_next().await? {
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await?;
        }

        // stop on the first file.
        let ps = rf
            .into_os_string()
            .into_string()
            .map_err(|e| Err(e));
        return Ok((filename.to_string(), ps?));
    }

    simple_error::bail!("no file saved")
}


// async fn receive_file(
//     session: Session,
//     config: web::Data<Config>,
//     mut payload: Multipart,
// ) -> HttpResponse {
//     let result = save(session, config, payload).await;

//     match result {
//         Ok(success_value) => HttpResponse::Ok().json(success_value),
//         Err(error) => HttpResponse::InternalServerError().body(format!("{:?}", error)),
//     }
// }

// fn create_insert_stmt(packet: &[u8]) -> String {
//     format!("insert into level0blobs (uuid, createdate, metadata, packet) values (?, ?, ?, ZEROBLOB({}))", packet.len())
// }

// async fn save(
//     session: Session,
//     config: web::Data<Config>,
//     mut payload: Multipart,
// ) -> Result<(), Box<dyn Error>> {
//     let conn = database::open_connection(config.db.as_path())?;

//     let (metadata, packet) = extract_files(payload).await?;
//     let uuid = Uuid::new_v4().to_string();
//     let now = util::now()?;

//     // write to sqlite.
//     let insert_stmt = create_insert_stmt(&packet);
//     conn.execute(insert_stmt.as_str(), params![uuid, now, metadata])?;

//     // Get the row id off the BLOB we just inserted.
//     let rowid = conn.last_insert_rowid();
//     // Open the BLOB we just inserted for IO.
//     let mut blob = conn.blob_open(DatabaseName::Main, "level0blobs", "packet", rowid, false)?;

//     blob.write(&packet);

//     Ok(())
// }

// #[derive(Debug, Deserialize, Serialize)]
// struct Metadata {
//     // fields go here
// }

// async fn extract_files(mut payload: Multipart) -> Result<(Metadata, Bytes), Box<dyn Error>> {
//     let mut metadata = Vec::new();
//     let mut packet = Vec::new();

//     while let Some(mut field) = payload.try_next().await? {
//         let content_disposition = field
//             .content_disposition()
//             .ok_or(simple_error::SimpleError::new("bad"))?;

//         let filename = content_disposition
//             .get_name()
//             .unwrap_or("fieldname not found");

//         if filename == "metadata" {
//             while let Some(chunk) = field.try_next().await? {
//                 metadata.extend(chunk);
//             }
//         } else if filename == "packet" {
//             while let Some(chunk) = field.try_next().await? {
//                 packet.extend(chunk);
//             }
//         }
//     }

//     let metadata = String::from_utf8(metadata)?;
//     let metadata: Metadata = serde_json::from_str(&metadata)?;
//     let packet = packet.into();

//     Ok((metadata, packet))
// }

// async fn receive_file_old(
//     session: Session,
//     config: web::Data<Config>,
//     mut payload: Multipart,
// ) -> HttpResponse {
//     match save_file_too(session, config, payload).await {
//         Ok(r) => HttpResponse::Ok().json(r),
//         Err(e) => return HttpResponse::InternalServerError().body(format!("{:?}", e)),
//     }
// }

// async fn save_file_too(
//     session: Session,
//     config: web::Data<Config>,
//     mut payload: Multipart,
// ) -> Result<(), Box<dyn Error>> {
//     let conn = database::open_connection(config.db.as_path())?;
//     let tp = config.file_tmp_path.clone();
//     let (name, fp) = save_file(&tp, payload).await?;
//     Ok(())
// }

// async fn save_file(
//     to_dir: &Path,
//     mut payload: Multipart,
// ) -> Result<(String, String), ExtractError> {
//     // iterate over multipart stream

//     // ONLY SAVING THE FIRST FILE
//     while let Some(mut field) = payload.try_next().await? {
//         // A multipart/form-data stream has to contain `content_disposition`
//         let content_disposition = field
//             .content_disposition()
//             .ok_or(simple_error::SimpleError::new("bad"))?;

//         let filename = content_disposition
//             .get_filename()
//             .unwrap_or("filename not found");

//         let wkfilename = Uuid::new_v4().to_string();

//         let mut filepath = to_dir.to_path_buf();
//         filepath.push(wkfilename);

//         let rf = filepath.clone();

//         // File::create is blocking operation, use threadpool
//         let mut f = web::block(|| std::fs::File::create(filepath)).await?;

//         // Field in turn is stream of *Bytes* object
//         while let Some(chunk) = field.try_next().await? {
//             // filesystem operations are blocking, we have to use threadpool
//             f = web::block(move || f.write_all(&chunk).map(|_| f)).await?;
//         }

//         // stop on the first file.
//         let ps = rf
//             .into_os_string()
//             .into_string()
//             .map_err(|osstr| simple_error!("couldn't convert filename to string"));
//         return Ok((filename.to_string(), ps?));
//     }

//     simple_error::bail!("no file saved")
// }
