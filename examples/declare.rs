use std::ops::Bound;

use futures_util::stream::StreamExt;
use morseldb::{executor::tokio::TokioExecutor, morsel_record, Projection, DB};

// Tips: must be public
#[morsel_record]
pub struct User {
    #[primary_key]
    name: String,
    email: Option<String>,
    age: u8,
}

#[tokio::main]
async fn main() {
    let db = DB::<User, _>::new("./db_path/users".into(), TokioExecutor::default())
        .await
        .unwrap();

    {
        // morseldb supports transaction
        let mut txn = db.transaction().await;

        // set with owned value
        txn.set(User {
            name: "Alice".into(),
            age: 22,
            email: Some("alice@gmail.com".into()),
        });

        // get from primary key
        let name = "Alice".into();
        let user = txn.get(&name, Projection::All).await.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().get().age, Some(22));

        let upper = "Blob".into();
        let mut scan = txn
            .scan((Bound::Included(&name), Bound::Excluded(&upper)))
            .await
            .projection(vec![])
            .take()
            .await
            .unwrap();
        loop {
            let user = scan.next().await.transpose().unwrap();
            match user {
                Some(entry) => {
                    assert_eq!(
                        entry.value(),
                        Some(UserRef {
                            name: "Alice",
                            age: Some(22),
                            email: Some("alice@gmail.com")
                        })
                    );
                }
                None => break,
            }
        }
    }
}