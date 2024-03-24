use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(rename = "operationName", skip_serializing_if = "String::is_empty")]
    pub operation_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}
