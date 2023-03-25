use tonic::{transport::Server, Request, Response, Status};
use hello::say_server::{Say, SayServer};
use hello::{UploadFile,
            UploadResponse,
            DownloadFile,
            DownloadResponse,
            SayResponse,
            SayRequest};
use std::io::Write;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::fs;
use crate::db::{get_file, put_file};
mod hello;
mod db;

// defining a struct for our service

#[derive(Default)]
pub struct MySay {}

// implementing rpc for service defined in .proto
#[tonic::async_trait]
impl Say for MySay {
    // our rpc impelemented as function
    async fn send(&self,request:Request<SayRequest>)->Result<Response<SayResponse>,Status>{
        // returning a response as SayResponse message as defined in .proto
        Ok(Response::new(SayResponse{
            // reading data from request which is awrapper around our SayRequest message defined in .proto
            message:format!("hello {}",request.get_ref().name),
        }))
    }

    async fn upload(&self, request: tonic::Request<UploadFile>) -> Result<tonic::Response<UploadResponse>, tonic::Status> {
        print!("BytesLen: {}\n", request.get_ref().image.len());
        let path = format!("/Users/lukamacieszczak/CLionProjects/grpc_demo/src/{}", request.get_ref().name);

        let mut file = fs::OpenOptions::new()
            // .create(true) // To create a new file
            .write(true)
            .create(true)
            // either use the ? operator or unwrap since it returns a Result
            .open(path.clone())?;
        
        file.write_all(&request.get_ref().image).expect("TODO: panic message");

        let hash = put_file(path, request.get_ref().extension.clone()).await.expect("TODO: panic message");

        Ok(Response::new(UploadResponse{
            message:hash,
        }))
    }

    async fn download(&self, request: tonic::Request<DownloadFile>) -> Result<tonic::Response<DownloadResponse>, tonic::Status>{
        let file_data = get_file(request.get_ref().key.clone()).await.expect("TODO: panic message");
        let mut file = File::open(file_data.path).await?;

        let mut contents: Vec<u8> = vec![];
        file.read_to_end(&mut contents).await?;

        println!("len = {}", contents.len());

        Ok(Response::new(DownloadResponse{
            image: contents,
            extension: file_data.extension,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = "[::1]:50051".parse().unwrap();

    // creating a service
    let say = MySay::default();
    println!("Server listening on {}", addr);
    // adding our service to our server.
    Server::builder()
        .add_service(SayServer::new(say))
        .serve(addr)
        .await?;
    Ok(())
}