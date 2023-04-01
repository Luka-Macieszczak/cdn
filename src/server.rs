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
use crate::db::{get_file, get_info, put_file};
mod hello;
mod db;
mod server_constants;
use std::path::Path;
use std::ffi::OsStr;
use crate::server_constants::GRPC_CHANNEL;

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

    async fn upload(&self, request: Request<UploadFile>) -> Result<Response<UploadResponse>, Status> {
        print!("BytesLen: {}\n", request.get_ref().file.len());

        let mut file = fs::OpenOptions::new()
            // .create(true) // To create a new file
            .write(true)
            .create(true)
            // either use the ? operator or unwrap since it returns a Result
            .open(request.get_ref().path.clone())?;
        
        file.write_all(&request.get_ref().file).expect("Write error");

        Ok(Response::new(UploadResponse{
            message:request.get_ref().hash.clone(),
        }))
    }

    async fn download(&self, request: Request<DownloadFile>) -> Result<Response<DownloadResponse>, Status>{
        let file_data = get_file(request.get_ref().key.clone()).await.expect("File read error");
        let mut file = File::open(file_data.path).await?;

        let mut contents: Vec<u8> = vec![];
        file.read_to_end(&mut contents).await?;

        println!("len = {}", contents.len());

        Ok(Response::new(DownloadResponse{
            file: contents,
            extension: file_data.extension,
            name: file_data.name
        }))
    }
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // defining address for our service
    let addr = GRPC_CHANNEL.parse().unwrap();

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

#[cfg(test)]
mod tests {
    use crate::get_extension_from_filename;
    #[test]
    fn file_extension_test(){
        assert_eq!(get_extension_from_filename("abc.gz"), Some("gz"));
    }
}