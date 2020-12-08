{% assign name = crate_name | remove: "_service" %}
{% assign plural_name = name | append: "s" %}
{% assign pascal = name | pascal_case %}

use dotenv::dotenv;
use log::{debug, warn};
use std::sync::Arc;

use tokio::sync::mpsc;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod api;
mod data;
mod db;

use crate::db::DataSources;
use {{crate_name}}::{{name}}_service_server::{{pascal}}Service, {{pascal}}ServiceServer};
use {{crate_name}}::{
    Delete{{pascal}}Request, Get{{pascal}}Request,
    List{{pascal}}sRequest, {{pascal}}, Update{{pascal}}Request
};

pub mod {{name}} {
    tonic::include_proto!("cosm.{{name}}");
}

#[derive(Clone)]
pub struct {{pascal}}s {
    data_sources: Arc<DataSources>,
}

#[tonic::async_trait]
impl {{pascal}}Service for {{pascal}}s {
    async fn create_{{name}}(
        &self,
        request: Request<{{pascal}}>,
    ) -> Result<Response<{{pascal}}>, tonic::Status> {
        let item = request.into_inner();
        api::items::create_one(&self.data_sources.{{plural_name}}, item).await
    }

    async fn get_{{name}}(
        &self,
        request: Request<Get{{pascal}}Request>,
    ) -> Result<tonic::Response<{{pascal}}>, tonic::Status> {
        warn!("Get{{pascal}} = {:?}", request);

        api::items::get_by_id(&self.data_sources.{{plural_name}}, &request.get_ref().id).await
    }

    type List{{pascal}}sStream = mpsc::Receiver<Result<{{pascal}}, Status>>;
    async fn list_{{plural_name}}(
        &self,
        request: Request<List{{pascal}}sRequest>,
    ) -> Result<tonic::Response<Self::List{{pascal}}sStream>, tonic::Status> {
        debug!("List{{pascal}}s = {:?}", request);

        let request = request.get_ref();
        if request.limit > 100 {
            return Err(Status::invalid_argument("Maximum number of items is 100"));
        }

        return Ok(api::items::stream(&self.data_sources.{{plural_name}}, request).await);
    }

    async fn update_{{name}}(
        &self,
        request: tonic::Request<Update{{pascal}}Request>,
    ) -> Result<tonic::Response<{{pascal}}>, tonic::Status> {
        warn!("Update{{pascal}} = {:?}", request);

        let request = request.get_ref();
        api::items::update_one(&self.data_sources.{{plural_name}}, request).await
    }

    async fn delete_{{name}}(
        &self,
        request: tonic::Request<Delete{{pascal}}Request>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        warn!("Delete{{pascal}} = {:?}", request);

        api::items::delete_by_id(&self.data_sources.{{plural_name}}, &request.get_ref().id)
            .await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    let addr = "0.0.0.0:10000".parse().unwrap();

    warn!("{{pascal}}Service listening on: {}", addr);
    let {{plural_name}} = {{pascal}}s {
        data_sources: db::connect().await,
    };

    let svc = {{pascal}}ServiceServer::new({{plural_name}});

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
