{% assign name = crate_name | remove: "_service" %}{% assign pascal = name | pascal_case %}

use log::warn;
use tonic::transport::Server;

use crate::db;
use crate::models::{items::Service, {{name}}_service_server::{{pascal}}ServiceServer};

pub(crate) async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:10000".parse().unwrap();

    warn!("{{pascal}}Service listening on: {}", addr);

    let service = Service {
        data_sources: db::connect().await,
    };

    Server::builder()
        .add_service({{pascal}}ServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}
