import type { CreateResourceRequest, Resource, SearchQuery, UpdateResourceRequest } from "@/types/api";
import service from "@/utils/request";

export const resourceApi = {
  // 分页搜索
  list: (params: SearchQuery) => 
    service.get<Resource[]>('/resources', { params }),

  // 创建
  create: (data: CreateResourceRequest) => 
    service.put<Resource>('/resource', data),

  // 更新
  update: (id: string, data: UpdateResourceRequest) => 
    service.post<Resource>(`/resource/${id}`, data),

    // 更新
  delete: (id: string) => 
    service.delete<undefined>(`/resource/${id}`),
};