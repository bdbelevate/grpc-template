use futures::future;
use futures::stream::StreamExt;
use log::{debug, error, info, warn};
use mongodb::{
    bson::{doc, from_bson, to_bson, Bson},
    options::{FindOptions, UpdateOptions},
    Collection,
};
use prost_types::FieldMask;
use rust_utils_lib::time_utils::current_timestamp;
use tokio::sync::mpsc;
use tonic::{Code, Response, Status};
{% assign name = crate_name | remove: "_service" %}{% assign pascal = name | pascal_case %}
use crate::db::id::{with_bson, ID};
use crate::{{name}}::{List{{pascal}}sRequest, {{pascal}}, Update{{pascal}}Request};
use crate::UpdateMode;

pub async fn create_one(
    collection: &Collection,
    mut item: {{pascal}},
) -> Result<Response<{{pascal}}>, tonic::Status> {
    if item.name == "" {
        return Err(Status::invalid_argument("name_required"));
    }
    let timestamp = current_timestamp();
    let version_metadata = doc! {
        "created_at": timestamp,
        "modified_at": timestamp,
        "created_by": user_id().to_string(),
        "modified_by": user_id().to_string(),
    };

    // create in db
    let serialized_member = to_bson(&item).map_err(|e| Status::unavailable(e.to_string()))?;
    if let Bson::Document(mut document) = serialized_member {
        // remove the id of this object so that mongo will generate
        document.remove("_id");
        document.insert("version_metadata", version_metadata);
        let insert_result = collection
            .insert_one(document, None)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        // convert id to a string
        item.id = with_bson(&insert_result.inserted_id);
        Ok(Response::new(item))
    } else {
        Err(Status::internal("INTERNAL ERROR"))
    }
}

pub async fn get_by_id(
    collection: &Collection,
    id: &str,
) -> Result<tonic::Response<{{pascal}}>, tonic::Status> {
    let id = ID::from_string(id)?;
    let filter = doc! { "_id": id.to_bson() };
    let some_item = collection.find_one(filter, None).await.map_err(|_| {
        error!("an error occurred");
        Status::internal("DATABASE ERROR")
    })?;

    match some_item {
        Some(doc) => {
            debug!("item: {:?}", doc);
            let item: {{pascal}} = from_bson(Bson::Document(doc))
                .map_err(|e| Status::internal(e.to_string()))?;
            Ok(Response::new(item))
        }
        None => Err(Status::not_found("NOT FOUND")),
    }
}

pub async fn stream(
    collection: &Collection,
    request: &List{{pascal}}sRequest,
) -> Response<mpsc::Receiver<Result<{{pascal}}, Status>>> {
    let (mut tx, rx) = mpsc::channel(100);

    let options = FindOptions::builder()
        .skip(Some(request.start as i64))
        .limit(Some(request.limit as i64))
        .build();

    let query = super::query::get_list_query(request);

    let total_items = collection
        .count_documents(query.clone(), None)
        .await
        .unwrap_or(0);
    let mut trailers = Status::new(Code::Ok, "complete");
    let map = trailers.metadata_mut();
    map.insert("total_items", total_items.to_string().parse().unwrap());

    let cursor_result = collection.find(query, options).await;

    if let Ok(cursor) = cursor_result {
        cursor
            .then(|c| match c {
                Ok(doc) => {
                    let item_result: Option<{{pascal}}> = from_bson(Bson::Document(doc)).map_or_else(
		    	|e| {
                            info!("Parse error: {:?}", e);
                            None
                        },
                        Some,
		    );
                    future::ready(item_result)
                }
                Err(_) => future::ready(None),
            })
            .fold(tx.clone(), |mut tx, some_item| async move {
                if let Some(item) = some_item {
                    debug!("item: {:?}", item);
                    tx.send(Ok(item.clone())).await.unwrap();
                }
                tx
            })
            .await;
        tx.send(Err(trailers)).await.unwrap();
    } else {
        tx.send(Err(Status::internal("DATABASE ERROR")))
            .await
            .unwrap();
    }
    Response::new(rx)
}

pub async fn update_one(
    collection: &Collection,
    request: &Update{{pascal}}Request,
    mode: UpdateMode,
) -> Result<tonic::Response<{{pascal}}>, tonic::Status> {
    let mut request = request.clone();
    let options = UpdateOptions::builder()
        .upsert(match mode {
            UpdateMode::Upsert => true,
            UpdateMode::Update => false,
        })
        .build();

    // If upsert mode all fields should be provided.
    if mode == UpdateMode::Upsert {
        request.mask = Some(FieldMask {
            paths: vec![
                "name".to_string(),
                "description".to_string(),
                "project_ids".to_string(),
                "{{name}}_type".to_string(),
            ],
        });
    }

    if request.mask.is_none() {
        return Err(Status::invalid_argument(
            "fieldmask is required for updating object",
        ));
    }
    if let Some(object) = request.object {
        let object_id = object.id.clone();
        let id = ID::from_string(object_id.clone())?;

        let query = doc! { "_id": id.to_bson() };

	// update if there's a mask and paths
	if let Some(mask) = &request.mask {
	    if !mask.paths.is_empty() {
	        let doc = mask.paths.iter().fold(doc! {}, |mut doc, path| {
	            match path.as_str() {
	                "name" => doc.insert("name", request.name.to_owned()),
	                "description" => doc.insert("description", request.description.to_owned()),
	                "project_ids" => doc.insert("project_ids", request.project_ids.to_owned()),
	                "{{name}}_type" => doc.insert("{{name}}_type", request.{{name}}_type.to_owned()),
	                _ => {
	                    warn!("Path: {} is not supported", path);
	                    None
	                }
	            };
	            doc
	        });
                // add additional metadata
                doc.insert("version_metadata.modified_at", current_timestamp());
                doc.insert("version_metadata.modified_by", String::from(user_id()));
                let mut upsert_overrides = doc! {};
                upsert_overrides.insert("version_metadata.created_at", current_timestamp());
                upsert_overrides.insert("version_metadata.created_by", String::from(user_id()));
            	let result = collection
                    .update_one(
                        query,
                        doc! { "$set": doc, "$setOnInsert": upsert_overrides },
                        options,
                    )
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;
                debug!("Update result: {:?}", result);
            }
    	}

        // get the updated object
        get_by_id(collection, &object_id).await
    } else {
        Err(Status::invalid_argument("Object is required"))
    }
}

pub async fn delete_by_id(
    collection: &Collection,
    id: &str,
) -> Result<tonic::Response<()>, tonic::Status> {
    let id = ID::from_string(id)?;
    let query = doc! { "_id": id.to_bson() };

    let _ = collection
        .delete_one(query, None)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

    Ok(Response::new(()))
}

/**
   TODO: Once we can get the User ID from the auth token, this method
   should go away and be replaced by a (library) method that extracts
   the user ID from the auth token.
*/
fn user_id() -> &'static str {
    "placeholder_user_id"
}
