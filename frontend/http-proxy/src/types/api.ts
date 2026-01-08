// types/api.ts
export interface ApiResponse<T = any> {
  code: number;
  data: T;
  message: string;
}

/**
 * 资源类型枚举
 * 对应 Rust 中的 ResourceType
 */
export enum ResourceType {
  S3 = "S3",
  PROXY = "PROXY",
  SFTP = "SFTP",
  // 以后可以方便地添加其他类型，如 REDIS = "REDIS"
}

/**
 * 核心资源实体
 * 对应 Rust 中的 Resource 结构体
 */
export interface Resource {
  id: string;
  name: string;
  kind: ResourceType;
  config: string;
  created_at: string; // 保持下划线
}
/**
 * 创建资源请求
 */
export interface CreateResourceRequest {
  name: string;
  kind: ResourceType;
  config: string;
}

/**
 * 更新资源请求
 * 对应 Rust 中的 Option<T>，在 TS 中体现为可选属性 '?'
 */
export interface UpdateResourceRequest {
  name?: string;
  kind?: ResourceType;
  config?: string;
}

/**
 * 搜索与分页查询
 */
export interface SearchQuery {
  name?: string;
  page?: number;
  page_size?: number; // 注意：Rust 中是 page_size，如果后端没设 rename，这里需改为 page_size
}