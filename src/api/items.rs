use futures::future;
use futures::stream::StreamExt;
use log::{debug, warn};
use mongodb::{
    bson::{doc, from_bson, to_bson, Bson, Document},
    options::FindOptions,
    Collection,
};
use tokio::sync::mpsc;
use tonic::{Response, Status};
{% assign name = crate_name | remove: "_service" %}{% assign pascal = name | pascal_case %}
use crate::api::get_timestamp;
use crate::db::id::{with_bson, ID};
use crate::{{name}}::{List{{pascal}}sRequest, {{pascal}}, Update{{pascal}}Request};

pub async fn create_one(
    collection: &Collection,
    mut item: {{pascal}},
) -> Result<Response<{{pascal}}>, tonic::Status> {
    if item.name == "" {
        return Err(Status::invalid_argument("name_required"));
    }
    item.created_at = Some(get_timestamp());

    // create in db
    let serialized_member = to_bson(&item).map_err(|e| Status::unavailable(e.to_string()))?;
    if let Bson::Document(mut document) = serialized_member {
        // remove the id of this object so that mongo will generate
        document.remove("_id");
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
        println!("an error occurred");
        Status::internal("DATABASE ERROR")
    })?;

    match some_item {
        Some(doc) => {
            println!("item: {:?}", doc);
            let item: {{pascal}} = from_bson(Bson::Document(doc.to_owned()))
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

    let ignored_ids = request.ignored_ids.iter().fold(vec![], |mut acc, id| {
        let id = ID::from_string(id);
        if let Ok(id) = id {
            acc.push(id.to_bson());
        }
        acc
    });
    let query = doc! {
        "_id": { "$nin": ignored_ids },
    };

    let filtered_types: Vec<i32> = request
        .{{name}}_types
        .iter()
        .map(|f| f.clone())
        .filter(|f| *f >= 0)
        .collect();
    if filtered_types.len() > 0 {
        query.insert("{{name}}_type", doc! { "$in": filtered_types });
    }

    if request.search_term.len() >= 2 {
        let search_doc: Vec<Document> = vec![
            doc! { "name": { "$regex": format!("{}", request.search_term), "$options": "i" } },
            doc! { "description": { "$regex": format!("{}", request.search_term), "$options": "i" } },
        ];
        query.insert("$or", search_doc);
    }

    let cursor_result = collection.find(query, options).await;

    if let Ok(cursor) = cursor_result {
        cursor
            .then(|c| match c {
                Ok(doc) => {
                    let item_result: Option<{{pascal}}> =
                        from_bson(Bson::Document(doc.to_owned()))
                            .map_or_else(|_| None, |x| Some(x));
                    future::ready(item_result)
                }
                Err(_) => future::ready(None),
            })
            .fold(tx, |mut tx, some_item| async move {
                if let Some(item) = some_item {
                    debug!("item: {:?}", item);
                    tx.send(Ok(item.clone())).await.unwrap();
                }
                tx
            })
            .await;
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
) -> Result<tonic::Response<{{pascal}}>, tonic::Status> {
    let id = ID::from_string(&request.id)?;
    let query = doc! { "_id": id.to_bson() };

    // update if there's a mask and paths
    if let Some(mask) = &request.mask {
        if mask.paths.len() > 0 {
            let doc = mask.paths.iter().fold(doc! {}, |mut doc, path| {
                match path.as_str() {
                    "name" => doc.insert("name", request.name.to_owned()),
                    "description" => doc.insert("description", request.description.to_owned()),
                    "{{name}}_type" => doc.insert("{{name}}_type", request.{{name}}_type.to_owned()),
                    _ => {
                        warn!("Path: {} is not supported", path);
                        None
                    }
                };
                doc
            });
            let result = collection
                .update_one(query, doc! { "$set": doc }, None)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            debug!("Update result: {:?}", result);
        }
    }

    // get the updated object
    get_by_id(collection, &request.id).await
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
