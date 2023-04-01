use tokio_postgres::{Client, Socket, NoTls, Error, Connection};
use sha2::{Sha256, Digest};
use std::error::Error as OtherError;


const POSTGRES_CONFIG: &str = "host=localhost user=admin password=password dbname=CDN";

pub struct FileData {
    pub(crate) extension: String,
    pub(crate) path: String,
    pub(crate) name: String
}

/// name = name of uploaded file. Will not be modified, but will have a different path
/// directory_path = path to directory where file will be uploaded to
pub async fn get_info(name: String, directory_path: String) -> Result<(String, String, i32), Error>{
    // Get current highest index
    // Not particularly efficient, maybe can do better later
    // New id will be one above last
    let (client, connection) = tokio_postgres::connect(
        POSTGRES_CONFIG,
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS Images(ID Integer, \
        FilePath VarChar(500), Key VarChar(500) \
        Not Null Unique, type VarChar(100), name VarChar(500), PRIMARY KEY(ID))"
    ).await?;

    let mut id = 1;

    for row in client.query("SELECT id, filepath, key FROM images", &[]).await? {
        if id < row.get(0){
            id = row.get(0)
        }
        // Check here for duplicates TODO
    }

    id += 1;

    let mut sha256 = Sha256::new();
    sha256.update(name.clone());
    let hash = format!("{}{:X}", id, sha256.finalize());
    let path = format!("{}{}{}", directory_path, hash, name);

    println!("ID: {}\n Path: {}\n Hash: {}\n", id, path, hash);
    Ok((hash, path, id))
}

pub async fn put_file(id: i32, file_path: String, extension: String, name: String, hash: String) -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        POSTGRES_CONFIG,
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });


    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS Images(ID Integer, \
        FilePath VarChar(500), Key VarChar(500) \
        Not Null Unique, type VarChar(100), name VarChar(500), PRIMARY KEY(ID))"
    ).await?;

    // Insert new file
    let sql = format!("INSERT INTO Images (id, filepath, key, type, name) \
    VALUES ({}, '{}', '{}', '{}', '{}')",
    id, file_path, hash, extension, name);

    client.execute(
        sql.as_str(),
        &[],
    ).await?;

    Ok(())
}


pub async fn get_file(key: String) -> Result<FileData, Error> {
    let (mut client, connection) = tokio_postgres::connect(
        POSTGRES_CONFIG,
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let sql = format!("SELECT filepath, type, name FROM images where key='{}'", key);
    let res = client.query(sql.as_str(), &[]).await?;
    let file_data = FileData { extension: res[0].get(1), path: res[0].get(0), name: res[0].get(2)};
    Ok(file_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file() {

        let result = put_file(10, String::from("Jeff"),
                              "".to_string(), "test".to_string(),
                              "test".to_string()).await;

        match result {
            Ok(val) => println!("written to DB"),
            Err(err) => {
                panic!("Error description: {}", err.to_string());
            }
        };
    }



    #[tokio::test]
    async fn test_get_file() {
        let key = String::from("26AD16AD763C14BA0E9B7DC5A036C76FA40BFECBBA2FB32C42BD82173BCF31670");
        let result = get_file(key).await;

        match result {
            Ok(val) => {
                println!("Path: {}", val.path);
                println!("Path: {}", val.extension);
            },
            Err(err) => {
                panic!("Error description: {}", err.to_string());
            }
        };
    }

    #[tokio::test]
    async fn test_data() {
        let (hash, path, id) = get_info("bob".to_string(),
                                        "jeff".to_string()).await.expect("");
        println!("Hash: {}", hash);
        println!("Path: {}", path);
        println!("ID: {}", id);
    }
}