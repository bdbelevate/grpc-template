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
use {{crate_name}}::{{crate_name}}_service_server::{SampleService, SampleServiceServer};
use orgs::{
    DeleteSampleRequest, GetSampleRequest,
    ListSamplesRequest, Sample, UpdateSampleRequest
};

pub mod {{crate_name}} {
    tonic::include_proto!("cosm.{{crate_name}}");
}

#[derive(Clone)]
pub struct Samples {
    data_sources: Arc<DataSources>,
}

#[tonic::async_trait]
impl SampleService for Samples {
    async fn create_sample(
        &self,
        request: Request<Sample>,
    ) -> Result<Response<Sample>, tonic::Status> {
        let sample = request.into_inner();
        api::samples::create_one(&self.data_sources.samples, sample).await
    }

    async fn get_organization(
        &self,
        request: Request<GetSampleRequest>,
    ) -> Result<tonic::Response<Sample>, tonic::Status> {
        warn!("GetSample = {:?}", request);

        api::samples::get_by_id(&self.data_sources.samples, &request.get_ref().id).await
    }

    type ListSamplesStream = mpsc::Receiver<Result<Sample, Status>>;
    async fn list_organizations(
        &self,
        request: Request<ListSamplesRequest>,
    ) -> Result<tonic::Response<Self::ListSamplesStream>, tonic::Status> {
        debug!("ListSamples = {:?}", request);

        let request = request.get_ref();
        if request.limit > 100 {
            return Err(Status::invalid_argument("Maximum number of items is 100"));
        }

        return Ok(api::samples::stream(&self.data_sources.samples, request).await);
    }

    async fn update_organization(
        &self,
        request: tonic::Request<UpdateSampleRequest>,
    ) -> Result<tonic::Response<Sample>, tonic::Status> {
        warn!("UpdateOrganziation = {:?}", request);

        let request = request.get_ref();
        api::samples::update_one(&self.data_sources.samples, request).await
    }

    async fn delete_organization(
        &self,
        request: tonic::Request<DeleteSampleRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        warn!("DeleteSample = {:?}", request);

        api::samples::delete_by_id(&self.data_sources.samples, &request.get_ref().id)
            .await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    let addr = "0.0.0.0:10000".parse().unwrap();

    warn!("SampleService listening on: {}", addr);
    let samples = Samples {
        data_sources: db::connect().await,
    };

    let svc = SampleServiceServer::new(samples);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
