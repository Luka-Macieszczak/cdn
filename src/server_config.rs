pub struct Config{
    pub upload_path: String,
    pub address: String
}

impl Config {
    pub fn new() -> Self {
        Self {
            upload_path: "/Users/lukamacieszczak/CLionProjects/grpc_demo/src/".parse().unwrap(),
            address: "[::1]:50051".parse().unwrap()
        }
    }
}