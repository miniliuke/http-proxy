// utils/request.ts
import axios, { type AxiosInstance, type AxiosRequestConfig, type AxiosResponse } from 'axios';

// 1. 创建实例
const service: AxiosInstance = axios.create({
  baseURL: 'http://127.0.0.1:3000/api/v1',
  timeout: 10000,
  headers: { 'Content-Type': 'application/json' }
});

// 2. 请求拦截器
service.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// 3. 响应拦截器
service.interceptors.response.use(
  (response: AxiosResponse) => {
    const res = response.data;
    
    // 根据后端约定的状态码判断
    if (res.code !== 200) {
      // 在这里可以进行全局提示（如使用 Element Plus 或 AntD 的 Message）
      console.error(res.message || 'Error');
      
      // 登录过期处理
      if (res.code === 401) {
        // 清除 Token 并跳转登录页
      }
      return Promise.reject(new Error(res.message || 'Error'));
    }
    return res.data;
  },
  (error) => {
    // 处理 HTTP 网络错误
    let message = '';
    const status = error.response?.status;
    switch (status) {
      case 401: message = '未授权，请登录'; break;
      case 403: message = '拒绝访问'; break;
      case 404: message = '请求地址错误'; break;
      case 500: message = '服务器内部错误'; break;
      default: message = '网络连接异常';
    }
    return Promise.reject(error);
  }
);



// 4. 导出通用请求工具函数
export const request = {
  get<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
    
    return service.get(url, config) as unknown as Promise<T>;
  },
  post<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    return service.post(url, data, config) as unknown as Promise<T>;
  },
  put<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    return service.put(url, data, config) as unknown as Promise<T>;
  },
  delete<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
    return service.delete(url, config) as unknown as Promise<T>;
  }
};

export default service;