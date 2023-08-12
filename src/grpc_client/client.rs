use proto::say_client::SayClient;
use http::{Method};
use axum_macros::debug_handler;
use axum::{routing::get,
           extract::State,
           routing::post,
           Router, Json,
           extract::Multipart,
           extract::DefaultBodyLimit,
           body::StreamBody,
           http::{header, StatusCode},
           response::{AppendHeaders, IntoResponse},
};
use bytes::Bytes;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};
use proto::SayRequest;
use crate::proto::{DownloadFile, UploadFile};
use std::env;
use std::fmt::Error;
use std::sync::mpsc::Sender;
use tokio_util::io::ReaderStream;
use crate::client_constants::{DIRECTORY_PATH, GRPC_CHANNEL};
use crate::db::{get_info, put_file, get_file_from_db};
use cdn::db;
use cdn::proto;
mod client_constants;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new().route("/", get(handler))
        .route("/test", get(test))
        .route("/download", post(download))
        .route("/upload", post(upload))
        .layer(DefaultBodyLimit::max(16777216))
        .layer(cors);
    // Address that server will bind to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 4041));

    axum::Server::bind(&addr)
        // Hyper server takes a make service.
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    send_hello(String::from("Luka")).await.expect("TODO: panic message");
    "Hello, world!"
}

async fn test() -> impl IntoResponse {
    //send_hello(payload.message).await.expect("TODO: panic message");

    //(StatusCode::OK, Json(user))
    "test"
}

#[derive(Serialize)]
struct UploadResponse {
    key: String,
    success: i32
}

#[debug_handler]
async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let mut bytes = Bytes::from_static(b"");
    let mut file_name = String::from("");
    let mut extension = String::from("");
    let mut seen_key = false;
    let mut res = UploadResponse {
        key: "".parse().unwrap(),
        success: 0
    };

    // Go through all parts of the request and set appropriate variables
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        match name.as_str() {
            "data" => bytes = field.bytes().await.unwrap(),
            "name" => file_name = field.text().await.unwrap().to_string(),
            "extension" => extension = field.text().await.unwrap().to_string(),
            "key" => {
                let key = field.text().await.unwrap().to_string();
                seen_key = true;

                if !verify_api_key(key){
                    return (StatusCode::OK, Json(res))
                }
            }
            _ => {}
        }
    }
    // Don't continue if there was no key
    if !seen_key{
        return (StatusCode::OK, Json(res))
    }

    let (hash, path, id) = get_info(file_name.clone(), String::from(DIRECTORY_PATH))
        .await.expect("File info error");

    put_file(id, path.clone(), extension.clone(),
             file_name.clone(), hash.clone()).await.expect("Database insert failure");

    println!("length is {} name is {} extension is {}", bytes.len(), file_name, extension);
    res.key = hash.clone();
    send_image(bytes.to_vec(), file_name, extension, hash, path, id).await.expect("GRPC upload failure");
    res.success = 1;
    (StatusCode::OK, Json(res))
}

#[derive(Deserialize)]
struct DownloadOptions {
    api_key: String,
    file_key: String,
}

async fn download(Json(payload): Json<DownloadOptions>) -> impl IntoResponse {
    let file_data = get_file(payload.file_key).await.expect("File retrieval failure");
    let headers = AppendHeaders([
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_data.name),
        )
    ]);

    // FIX THIS
    if !verify_api_key(payload.api_key){
        return (StatusCode::BAD_REQUEST, headers, vec![])
    }

    print!("\n\nFile extension: {}\n\n", file_data.extension);
    (StatusCode::OK, headers, file_data.file)
}


async fn send_hello(message: String) -> Result<(), Box<dyn std::error::Error>> {
    // creating a channel ie connection to server
    let channel = tonic::transport::Channel::from_static(GRPC_CHANNEL)
        .connect()
        .await?;
    // creating gRPC client from channel
    let mut client = SayClient::new(channel);
    // creating a new Request
    let request = tonic::Request::new(
        SayRequest {
            name:message,
        },
    );
    // sending request and waiting for response
    let response = client.send(request).await?.into_inner();
    println!("RESPONSE={:?}", response.message);
    Ok(())
}

async fn send_image(bytes: Vec<u8>, name: String, extension: String, hash: String, path: String, id: i32)
    -> Result<String, Box<dyn std::error::Error>> {
    // creating a channel ie connection to server
    print!("Bytes len: {}\n", bytes.len());

    let channel = tonic::transport::Channel::from_static(GRPC_CHANNEL)
        .connect()
        .await?;
    // creating gRPC client from channel
    let mut client = SayClient::new(channel);
    // creating a new Request
    let request = tonic::Request::new(
        UploadFile {
            file: bytes,
            extension,
            name,
            hash,
            path,
            id
        },
    );
    // sending request and waiting for response
    let response = client.upload(request).await?.into_inner();
    println!("RESPONSE={:?}", response.message);
    Ok(response.message)
}
#[derive(Serialize)]
struct FileData {
    pub(crate) file: Vec<u8>,
    pub(crate) extension: String,
    pub(crate) name: String
}

async fn get_file(key: String) -> Result<FileData, Box<dyn std::error::Error>> {
    let channel = tonic::transport::Channel::from_static(GRPC_CHANNEL)
        .connect()
        .await?;
    let file_info = get_file_from_db(key.clone()).await.expect("File read error");
    // creating gRPC client from channel
    let mut client = SayClient::new(channel);
    // creating a new Request
    let request = tonic::Request::new(
        DownloadFile {
            key,
            path: file_info.path,
            extension: file_info.extension.clone(),
            name: file_info.name.clone()
        },
    );
    // sending request and waiting for response
    let response = client.download(request).await?.into_inner();
    let file_data = FileData {file: response.file, extension: file_info.extension, name: file_info.name};
    Ok(file_data)
}

fn verify_api_key(key: String) -> bool {
    match env::var("API_KEY") {
        Ok(v) => return v == key,
        Err(e) => panic!("$not set ({})\n\n", e)
    }
}