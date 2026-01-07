use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use http::StatusCode;
use uuid::Uuid;

use crate::server::{
    api::AppState,
    application::resource::{CreateResourceRequest, SearchQuery, UpdateResourceRequest},
};

pub async fn create_resource(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateResourceRequest>,
) -> impl IntoResponse {
    match state.resource_service.create(payload).await {
        Ok(resource) => (StatusCode::OK, Json(resource)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_resource(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.resource_service.get(id).await {
        Ok(resource) => Json(resource).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn search_resources(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let size = params.page_size.unwrap_or(10).max(1);

    match state.resource_service.search(params.name, page, size).await {
        Ok(resources) => Json(resources).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_resource(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateResourceRequest>,
) -> impl IntoResponse {
    match state.resource_service.update(id, payload).await {
        Ok(resource) => Json(resource).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_resource(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.resource_service.delete(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
