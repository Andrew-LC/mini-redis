use bytes::Bytes;
use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;
    client.set("seven", Bytes::from(vec![7, 8])).await?;

    // Get key "hello"
    let result = client.get("hello").await?;
    let result2 = client.get("seven").await?;

    println!("got value from the server; result={:?}", result);
    println!("got value from the server; result={:?}", result2);

    Ok(())
}

