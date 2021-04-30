{% assign name = crate_name | remove: "_service" %}{% assign plural_name = name | append: "s" %}{% assign pascal = name | pascal_case %}

tonic::include_proto!("cosm.{{name}}");

use std::{str::FromStr, sync::Arc};

use {{name}}_service_server::{{pascal}}Service;
use log::{debug, warn};
use strum_macros::EnumString;
use tokio::sync::mpsc::Receiver;
use tonic::{Request, Response, Status};

use crate::api::items::{ create_one, delete_by_id, get_by_id, stream, update_one };
use crate::db::DataSources;

#[derive(Clone)]
pub(crate) struct Service {
    pub data_sources: Arc<DataSources>,
}

#[derive(PartialEq, EnumString, Debug)]
pub(crate) enum UpdateMode {
    Update,
    Upsert,
}

#[tonic::async_trait]
impl {{pascal}}Service for Service {
    type List{{pascal}}sStream = Receiver<Result<{{pascal}}, Status>>;
    
    async fn create_{{name}}(
        &self,
        request: Request<{{pascal}}>,
    ) -> Result<Response<{{pascal}}>, Status> {
        let item = request.into_inner();
        create_one(&self.data_sources.{{plural_name}}, item).await
    }

    async fn get_{{name}}(
        &self,
        request: Request<Get{{pascal}}Request>,
    ) -> Result<Response<{{pascal}}>, Status> {
        warn!("Get{{pascal}} = {:?}", request);

        get_by_id(&self.data_sources.{{plural_name}}, &request.get_ref().id).await
    }

    async fn list_{{plural_name}}(
        &self,
        request: Request<List{{pascal}}sRequest>,
    ) -> Result<Response<Self::List{{pascal}}sStream>, Status> {
        debug!("List{{pascal}}s = {:?}", request);

        let request = request.get_ref();
        if request.limit > 100 {
            return Err(Status::invalid_argument("Maximum number of items is 100"));
        }

        return Ok(stream(&self.data_sources.{{plural_name}}, request).await);
    }

    async fn update_{{name}}(
        &self,
        request: Request<Update{{pascal}}Request>,
    ) -> Result<Response<{{pascal}}>, Status> {
        warn!("Update{{pascal}} = {:?}", request);
        let meta_mode = request.metadata().get("mode");
        let mut mode = UpdateMode::Update;

        // Validate and set mode only when header exists
        // Needed for backwards compatibility
        if meta_mode.is_some() {
            mode = UpdateMode::from_str(meta_mode.unwrap().to_str().unwrap()).map_err(|e| {
                Status::invalid_argument(format!(
                    "Unable to parse metadata.mode {}. Possible values are Upsert or Update",
                    e.to_string()
                ))
            })?;
        }

        let request = request.get_ref();
        update_one(&self.data_sources.{{plural_name}}, request, mode).await
    }

    async fn delete_{{name}}(
        &self,
        request: Request<Delete{{pascal}}Request>,
    ) -> Result<Response<()>, Status> {
        warn!("Delete{{pascal}} = {:?}", request);

        delete_by_id(&self.data_sources.{{plural_name}}, &request.get_ref().id).await
    }
}
