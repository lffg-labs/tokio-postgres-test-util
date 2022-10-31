use std::sync::Arc;

use http_problem::prelude::*;
use tokio_postgres::Client;

#[async_trait::async_trait]
trait TestRepository {
    async fn data(&self) -> Result<i32>;
}

struct PostgresTestRepository {
    client: Arc<Client>,
}

#[async_trait::async_trait]
impl TestRepository for PostgresTestRepository {
    async fn data(&self) -> Result<i32> {
        let row = self.client.query_one("SELECT 1 + 1 as res;", &[]).await?;
        Ok(row.get("res"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = connect().await?;

    let test_repository = PostgresTestRepository { client };

    let res = test_repository.data().await?;
    println!("got {res}");

    Ok(())
}

async fn connect() -> Result<Arc<Client>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=luiz dbname=pg-example",
        tokio_postgres::NoTls,
    )
    .await?;

    tokio::spawn(async move {
        match connection.await {
            Ok(_) => println!("connection closed"),
            Err(e) => println!("connection error: {e}"),
        }
    });

    Ok(Arc::new(client))
}

#[cfg(test)]
mod tests;
