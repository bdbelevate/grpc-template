use mongodb::bson::{doc, Document};
{% assign name = crate_name | remove: "_service" %}{% assign pascal = name | pascal_case %}
use crate::category::&List{{pascal}}sRequest;
use crate::db::id::ID;

pub fn get_list_query(request: &List{{pascal}}sRequest) -> Document {
    let ignored_ids = request.ignored_ids.iter().fold(vec![], |mut acc, id| {
        let id = ID::from_string(id);
        if let Ok(id) = id {
            acc.push(id.to_bson());
        }
        acc
    });
    let mut query = doc! {
        "_id": { "$nin": ignored_ids },
    };
    if !request.project_ids.is_empty() {
        query.insert("project_ids", doc! { "$in": request.project_ids.clone() });
    }

    let filtered_types: Vec<i32> = request
        .{{name}}_types
        .iter()
        .copied()
        .filter(|f| *f >= 0)
        .collect();
    if !filtered_types.is_empty() {
        query.insert("{{name}}_type", doc! { "$in": filtered_types });
    }

    if request.search_term.len() >= 2 {
        let search_doc: Vec<Document> = vec![
            doc! { "name": { "$regex": &request.search_term, "$options": "i" } },
            doc! { "description": { "$regex": &request.search_term, "$options": "i" } },
        ];
        query.insert("$or", search_doc);
    }
    
    query
}
