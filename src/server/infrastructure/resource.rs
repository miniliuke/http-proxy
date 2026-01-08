use super::super::domain::resource::Result;
use sqlx::{Row, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    error::ProxyError,
    server::domain::resource::{Resource, ResourceRepository, ResourceType},
};

pub struct SqliteResourceRepository {
    pool: SqlitePool,
}

impl SqliteResourceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // 方便初始化表结构的方法
    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS resources (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                config TEXT NOT NULL,
                created_at DATETIME NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ResourceRepository for SqliteResourceRepository {
    async fn create(&self, resource: Resource) -> Result<Resource> {
        // SQLite 3.35+ 支持 RETURNING 子句，sqlx 默认支持
        let rec = sqlx::query_as::<_, Resource>(
            r#"
            INSERT INTO resources (id, name, kind, config, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, kind, config, created_at
            "#,
        )
        .bind(resource.id) // sqlx 会自动将 Uuid 转为 TEXT
        .bind(resource.name)
        .bind(resource.kind.to_string())
        .bind(resource.config)
        .bind(resource.created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(rec)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Resource>> {
        // SQLite 存储的是 String，手动处理 Enum 转换以防万一
        let row = sqlx::query("SELECT * FROM resources WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            let type_str: String = r.try_get("kind")?;
            // 如果 resource_type 列在数据库里可能存了脏数据，这里做个容错

            let resource_type =
                ResourceType::from_str(&type_str).map_err(|e| ProxyError::Common(e.to_string()))?;

            Ok(Some(Resource {
                id: r.try_get("id")?, // sqlx 自动把 TEXT 转回 Uuid
                name: r.try_get("name")?,
                kind: resource_type,
                config: r.try_get("config")?,
                created_at: r.try_get("created_at")?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn search(
        &self,
        name_query: Option<String>,
        kind: Option<ResourceType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Resource>> {
        let pattern = name_query
            .map(|s| format!("%{}%", s))
            .unwrap_or("%".to_string());

        // SQLite 的 LIKE 默认对 ASCII 字符不区分大小写
        // 注意：PostgreSQL 用的是 ILIKE，这里改回 LIKE
        let rows = sqlx::query(
            "SELECT * FROM resources WHERE name LIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut resources = Vec::new();
        for r in rows {
            let type_str: String = r.try_get("kind")?;
            resources.push(Resource {
                id: r.try_get("id")?,
                name: r.try_get("name")?,
                kind: ResourceType::from_str(&type_str).unwrap_or(ResourceType::S3),
                config: r.try_get("config")?,
                created_at: r.try_get("created_at")?,
            });
        }
        Ok(resources)
    }

    async fn update(&self, resource: Resource) -> Result<Option<Resource>> {
        let rec = sqlx::query_as::<_, Resource>(
            r#"
            UPDATE resources
            SET name = $1, kind = $2, config = $3, create_at = $4
            WHERE id = $5
            RETURNING id, name, kind, config, created_at
            "#,
        )
        .bind(resource.name)
        .bind(resource.kind.to_string())
        .bind(resource.config)
        .bind(resource.created_at)
        .bind(resource.id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(rec)
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM resources WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
