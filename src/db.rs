use tokio_postgres::{Client, NoTls, Error};
use sha2::{Sha256, Digest};
use std::error::Error as OtherError;

/**
Store file path to database
Generate unique hash for the file
maybe append index to front of sha256 encoding of path (should be unique)
*/

pub struct FileData {
    pub(crate) extension: String,
    pub(crate) path: String
}

pub async fn put_file(file_path: String, extension: String) -> Result<String, Error> {
    let (mut client, connection) = tokio_postgres::connect(
        "host=localhost user=admin password=password dbname=CDN",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });


    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS Images(ID Integer, \
        FilePath VarChar(100), Key VarChar(100) \
        Not Null Unique, type VarChar(100), PRIMARY KEY(ID))"
    ).await?;

    // Get current highest index
    // Not particularly efficient, maybe can do better later
    // New id will be one above last
    let mut id = 1;

    for row in client.query("SELECT id, filepath, key FROM images", &[]).await? {
        if id < row.get(0){
            id = row.get(0)
        }
        // Check here for duplicates
    }

    id += 1;

    let mut sha256 = Sha256::new();
    sha256.update(file_path.clone());
    let hash = format!("{}{:X}", id, sha256.finalize());

    // Insert new file
    let sql = format!("INSERT INTO Images (id, filepath, key, type) VALUES ({}, '{}', '{}', '{}')",
    id, file_path, hash, extension);

    client.execute(
        sql.as_str(),
        &[],
    ).await?;

    Ok(hash)
}


pub async fn get_file(key: String) -> Result<FileData, Error> {
    let (mut client, connection) = tokio_postgres::connect(
        "host=localhost user=admin password=password dbname=CDN",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let sql = format!("SELECT filepath, type FROM images where key='{}'", key);
    let res = client.query(sql.as_str(), &[]).await?;
    let file_data = FileData { extension: res[0].get(1), path: res[0].get(0)};
    Ok(file_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file() {

        let result = put_file(String::from("Jeff"), "".to_string()).await;

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
}