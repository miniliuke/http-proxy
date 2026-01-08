use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::ProxyError,
    server::domain::resource::{Resource, ResourceRepository, ResourceType},
};

#[derive(Deserialize)]
pub struct CreateResourceRequest {
    pub name: String,
    pub kind: ResourceType,
    pub config: String,
}

#[derive(Deserialize)]
pub struct UpdateResourceRequest {
    pub name: Option<String>,
    pub kind: Option<ResourceType>,
    pub config: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub name: Option<String>,
    pub kind: Option<ResourceType>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub struct ResourceService {
    repo: Arc<dyn ResourceRepository>,
}

impl ResourceService {
    pub fn new(repo: Arc<dyn ResourceRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, req: CreateResourceRequest) -> Result<Resource, ProxyError> {
        // 这里可以添加业务验证，例如校验 config 格式
        let resource = Resource::new(req.name, req.kind, req.config);
        self.repo.create(resource).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Resource, ProxyError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(ProxyError::Common0("NotFound"))
    }

    pub async fn search(
        &self,
        name: Option<String>,
        kind: Option<ResourceType>,
        page: i64,
        size: i64,
    ) -> Result<Vec<Resource>, ProxyError> {
        let offset = (page - 1) * size;
        self.repo.search(name, kind,size, offset).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateResourceRequest) -> Result<(), ProxyError> {
        let mut resource = self.get(id).await?;

        // 更新字段
        if let Some(name) = req.name {
            resource.name = name;
        }
        if let Some(rtype) = req.kind {
            resource.kind = rtype;
        }
        if let Some(cfg) = req.config {
            resource.config = cfg;
        }

        self.repo.update(resource).await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ProxyError> {
        self.repo.delete(id).await?;
        Ok(())
    }
}
