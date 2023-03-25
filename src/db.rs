use tokio_postgres::{Client, NoTls, Error};
use sha2::{Sha256, Digest};
use std::error::Error as OtherError;
/**
Store file path to database
Generate unique hash for the file
maybe append index to front of sha256 encoding of path (should be unique)
*/
pub async fn put_file(file_path: String) -> Result<String, Error> {
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
        "CREATE TABLE IF NOT EXISTS Images(ID Integer, FilePath VarChar(100), Key VarChar(100) Not Null Unique, PRIMARY KEY(ID))"
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
    let sql = format!("INSERT INTO Images (id, filepath, key) VALUES ({}, '{}', '{}')",
    id, file_path, hash);

    client.execute(
        sql.as_str(),
        &[],
    ).await?;

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file() {

        let result = put_file(String::from("Jeff"));

        match result {
            Result::Ok(val) => println!("written to DB"),
            Result::Err(err) => {
                panic!("Error description: {}", err.to_string());
            }
        };
    }
}