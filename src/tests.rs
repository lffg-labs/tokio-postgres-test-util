use std::{future::Future, panic::AssertUnwindSafe, sync::Arc};

use futures_util::FutureExt;
use tokio_postgres::Client;

use super::{connect, PostgresTestRepository, TestRepository};

#[tokio::test]
async fn test_repository_data_should_return_2() {
    with_client(|client| async move {
        let repo = PostgresTestRepository { client };
        let data = repo.data().await.unwrap();
        assert_eq!(data, 2);
    })
    .await;
}

#[tokio::test]
async fn test_isolated() {
    const QUERY: &str = "create table isolated_test ( id int primary key );";

    let panic = AssertUnwindSafe(with_client(|client| async move {
        let fst = client.execute(QUERY, &[]).await;
        assert!(fst.is_ok());

        let snd = client.execute(QUERY, &[]).await;
        assert!(snd.is_err());

        panic!("explode now");
    }))
    .catch_unwind()
    .await;

    assert!(panic.is_err());

    with_client(|client| async move {
        let fst = client.execute(QUERY, &[]).await;
        assert!(fst.is_ok());
    })
    .await;
}

async fn with_client<Fn, Fut>(f: Fn)
where
    Fn: FnOnce(Arc<Client>) -> Fut,
    Fut: Future<Output = ()>,
{
    let client = connect().await.unwrap();
    client.execute("BEGIN;", &[]).await.expect("tx begin");

    let res = AssertUnwindSafe(f(client.clone())).catch_unwind().await;

    client.execute("ROLLBACK;", &[]).await.expect("tx rollback");

    if let Err(panic) = res {
        std::panic::resume_unwind(panic);
    }
}
