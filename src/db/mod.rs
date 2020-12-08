use log::warn;
use mongodb::Client;
use mongodb::Collection;
use std::env;
use std::sync::Arc;

pub mod id;

#[derive(Clone)]
pub struct DataSources {
    pub samples: Collection,
}

pub async fn connect() -> Arc<DataSources> {
    // set up database connection pool
    let mongo_url = env::var("MONGO_URL").expect("MONGO_URL must be set");
    let mongo_db_name = env::var("MONGO_DB_NAME").expect("MONGO_DB_NAME must be set");

    warn!("Connecting to database");
    let db = Client::with_uri_str(&mongo_url)
        .await
        .expect("Failed to initialize client.")
        .database(&mongo_db_name);

    Arc::new(DataSources {
        samples: db.collection("samples"),
    })
}
