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

    //start the grpc server
    server::run().await
}
