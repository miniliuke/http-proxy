use std::{error::Error, str::FromStr};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::error::ProxyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] // API JSON中使用: DATABASE, API_GATEWAY
pub enum ResourceType {
    S3,
    Proxy,
    SFTP,
}

impl TryFrom<String> for ResourceType {
    // 这里使用 strum 定义的错误类型，或者你可以自定义
    type Error = ProxyError;

    fn try_from(s: String) -> Result<Self> {
        // 这里的 &s 将 String 借用为 &str，然后调用 strum 生成的 from_str
        ResourceType::from_str(&s).map_err(|e| ProxyError::Common(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: Uuid,
    pub name: String,
    #[sqlx(try_from = "String")]
    pub kind: ResourceType,
    pub config: String,
    pub created_at: DateTime<Utc>,
}

impl Resource {
    // 领域逻辑：创建一个新资源
    pub fn new(name: String, resource_type: ResourceType, config: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            kind: resource_type,
            config,
            created_at: Utc::now(),
        }
    }
}

// 定义统一的错误类型，避免 Application 层感知 DbErr
pub type Result<T> = std::result::Result<T, ProxyError>;

#[async_trait::async_trait]
pub trait ResourceRepository: Send + Sync {
    async fn create(&self, resource: Resource) -> Result<Resource>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Resource>>;
    // 搜索：支持按名称模糊搜索
    async fn search(
        &self,
        name_query: Option<String>,
        kind: Option<ResourceType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Resource>>;
    // 更新：返回 Option，如果版本不匹配返回 None 或报错
    async fn update(&self, resource: Resource) -> Result<Option<Resource>>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}
