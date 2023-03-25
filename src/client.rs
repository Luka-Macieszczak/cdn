use hello::say_client::SayClient;
use http::{Method};
use axum::{routing::get,
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
use hello::SayRequest;
use crate::hello::{DownloadFile, UploadFile};
use std::env;
use tokio_util::io::ReaderStream;

mod hello;
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
    //        Router::new().route("/", get(|| async { "Hello, world!" }));
    // Address that server will bind to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 4041));

    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
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

async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let mut bytes = Bytes::from_static(b"hello");
    let mut file_name = String::from("");
    let mut extension = String::from("");
    let mut seen_key = false;
    let mut res = UploadResponse {
        key: "".parse().unwrap(),
        success: 0
    };
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        /// Clean this whole block
        /// I might be sick
        if name == "data" {
            bytes = field.bytes().await.unwrap();
        }
        else if name == "name" {
            file_name = field.text().await.unwrap().to_string();
        }
        else if name == "extension" {
            extension = field.text().await.unwrap().to_string();
        }
        else if name == "key" {
            let key = field.text().await.unwrap().to_string();
            match env::var("API_KEY") {
                Ok(v) => {
                    seen_key = true;
                    if v != key{
                        return (StatusCode::OK, Json(res))
                    }
                },
                Err(e) => panic!("$not set ({})\n\n", e)
            }
        }
    }
    // Don't continue if there was no key
    if !seen_key{
        return (StatusCode::OK, Json(res))
    }
    println!("length is {} name is {} extension is {}", bytes.len(), file_name, extension);
    res.key = send_image(bytes.to_vec(), file_name, extension).await.expect("TODO: panic message");
    res.success = 1;
    (StatusCode::OK, Json(res))
}

async fn download(json: Json) -> impl IntoResponse {
    // `File` implements `AsyncRead`
    let file = match tokio::fs::File::open("Cargo.toml").await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let image_data = get_image()

    let headers = AppendHeaders([
        ("content-type", "text/toml; charset=utf-8"),
        (
            "content-disposition",
            "attachment; filename=\"Cargo.toml\"",
        ),
    ]);

    Ok((headers, body))
}


async fn send_hello(message: String) -> Result<(), Box<dyn std::error::Error>> {
    // creating a channel ie connection to server
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
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

async fn send_image(bytes: Vec<u8>, name: String, extension: String) -> Result<String, Box<dyn std::error::Error>> {
    // creating a channel ie connection to server
    print!("Bytes len: {}\n", bytes.len());

    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;
    // creating gRPC client from channel
    let mut client = SayClient::new(channel);
    // creating a new Request
    let request = tonic::Request::new(
        UploadFile {
            image: bytes,
            extension,
            name
        },
    );
    // sending request and waiting for response
    let response = client.upload(request).await?.into_inner();
    println!("RESPONSE={:?}", response.message);
    Ok(response.message)
}

struct ImageData {
    pub(crate) image: Vec<u8>,
    pub(crate) extension: String
}

async fn get_image(key: String) -> Result<ImageData, Box<dyn std::error::Error>> {
    // creating a channel ie connection to server
    print!("Bytes len: {}\n", bytes.len());

    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;
    // creating gRPC client from channel
    let mut client = SayClient::new(channel);
    // creating a new Request
    let request = tonic::Request::new(
        DownloadFile {
            key
        },
    );
    // sending request and waiting for response
    let response = client.upload(request).await?.into_inner();
    let image_data = ImageData {image: response.image, extension: response.extension};
    Ok(image_data)
}
