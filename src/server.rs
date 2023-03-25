use tonic::{transport::Server, Request, Response, Status};
use hello::say_server::{Say, SayServer};
use hello::{SayResponse, SayRequest};
use hello::{UploadFile, UploadResponse};
use std::io::Write;
use std::fs;
use crate::db::put_file;

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


        // Need to spawn new thread for db functions
        //put_file(path).await.expect("TODO: panic message");

        Ok(Response::new(UploadResponse{
            // reading data from request which is awrapper around our SayRequest message defined in .proto
            message:format!("hello {}", "Bob"),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = "[::1]:50051".parse().unwrap();


    put_file(String::from("Joe")).await.expect("TODO: panic message");
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