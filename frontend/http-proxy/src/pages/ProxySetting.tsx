import { resourceApi } from '@/api/resource';
import { TaskTemplateForm, type TaskFieldGroup } from '@/components/task-template-form';
import { Toaster } from '@/components/ui/sonner';
import { ResourceType } from '@/types/api'; // 确保这里面定义了 PROXY 枚举值，例如 ResourceType.PROXY
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';

// --- 3. Proxy 配置页面实现 ---

// 定义 Proxy 表单数据的类型
interface ProxyConfigState {
    resourceName: string;
    address: string;  // 代理地址，如 http://127.0.0.1:7890
    username?: string; // 可选
    password?: string; // 可选
}

const ProxyConfigPage: React.FC = () => {
    const navigate = useNavigate();
    
    // 状态管理
    const [formData, setFormData] = useState<ProxyConfigState>({
        resourceName: '',
        address: '',
        username: '',
        password: '',
    });

    // 通用的变更处理函数
    const handleChange = (key: keyof ProxyConfigState, value: string) => {
        setFormData((prev) => ({
            ...prev,
            [key]: value,
        }));
    };

    // 提交处理
    const handleSubmit = async () => {
        // 1. 简单校验
        // 账号密码为可选，所以只校验名称和地址
        if (!formData.resourceName || !formData.address) {
            toast.warning("请填写资源名称和代理地址");
            return;
        }

        try {
            // 2. 提交 API
            await resourceApi.create({
                name: formData.resourceName,
                // 假设 ResourceType 枚举中有 PROXY 类型，如果没有请自行添加或替换为字符串
                kind: ResourceType.PROXY, 
                config: JSON.stringify({
                    address: formData.address,
                    username: formData.username,
                    password: formData.password,
                }),
            });

            // 3. 成功逻辑
            toast.success("代理资源创建成功");
            navigate("/resources"); 

        } catch (error: any) {
            // 4. 失败逻辑
            console.error(error);
            toast.error("创建失败", {
                description: error.message || "服务器发生错误，请稍后再试",
                action: {
                    label: "重试",
                    onClick: () => handleSubmit(),
                },
            });
        }
    };

    // 辅助函数：生成样式统一的 Input
    const renderInput = (
        key: keyof ProxyConfigState,
        placeholder: string,
        type: string = 'text'
    ) => {
        return (
            <input
                type={type}
                value={formData[key] || ''} // 处理可选字段可能为 undefined 的情况
                onChange={(e) => handleChange(key, e.target.value)}
                placeholder={placeholder}
                style={{
                    padding: '8px',
                    border: '1px solid #ccc',
                    borderRadius: '4px',
                    width: '100%',
                    boxSizing: 'border-box'
                }}
            />
        );
    };

    // --- 核心配置：构建 Groups ---
    const configGroups: TaskFieldGroup[] = [
        {
            groupName: "基础信息",
            fields: [
                {
                    id: "resourceName",
                    label: "资源名称",
                    render: () => renderInput("resourceName", "给此代理配置起个名字"),
                },
            ],
        },
        {
            groupName: "代理服务器配置",
            fields: [
                {
                    id: "address",
                    label: "代理地址",
                    render: () => renderInput("address", "例如: http://127.0.0.1:7890 或 socks5://proxy.example.com:1080"),
                },
                {
                    id: "username",
                    label: "账号 (可选)",
                    render: () => renderInput("username", "如果代理不需要验证，请留空"),
                },
                {
                    id: "password",
                    label: "密码 (可选)",
                    render: () => renderInput("password", "请输入密码", "password"), // 密码类型
                },
            ],
        },
    ];

    return (
        <div style={{ background: '#fff', minHeight: '100vh', padding: '20px' }}>
            <Toaster position="top-right" richColors />
            <TaskTemplateForm
                groups={configGroups}
                onSubmit={handleSubmit}
                submitText="保存配置"
            />
        </div>
    );
};

export default ProxyConfigPage;