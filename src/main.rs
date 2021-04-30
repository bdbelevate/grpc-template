use dotenv::dotenv;

mod api;
mod db;
mod models;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // allow environment variables set in .env
    dotenv().ok();

    // logger
    env_logger::init();
    let addr = "0.0.0.0:10000".parse().unwrap();

    //start the grpc server
    server::run().await
}
