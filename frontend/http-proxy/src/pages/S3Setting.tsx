import { resourceApi } from '@/api/resource';
import { TaskTemplateForm, type TaskFieldGroup } from '@/components/task-template-form';
import { Toaster } from '@/components/ui/sonner';
import { ResourceType } from '@/types/api';
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';


// --- 2. S3 配置页面实现 ---

// 定义 S3 表单数据的类型
interface S3ConfigState {
    resourceName: string;
    endpoint: string;
    region: string;
    bucket: string;
    accessKey: string;
    secretKey: string;
}

const S3ConfigPage: React.FC = () => {
    const navigate = useNavigate();
    // 状态管理：所有数据由父组件（当前页面）控制
    const [formData, setFormData] = useState<S3ConfigState>({
        resourceName: '',
        endpoint: '',
        region: 'us-east-1', // 可以设置默认值
        bucket: '',
        accessKey: '',
        secretKey: '',
    });

    // 通用的变更处理函数
    const handleChange = (key: keyof S3ConfigState, value: string) => {
        setFormData((prev) => ({
            ...prev,
            [key]: value,
        }));
    };

    // 提交处理
    const handleSubmit = async () => {
        // 1. 简单校验
        if (!formData.resourceName || !formData.bucket) {
            // 建议这里也把 alert 换成 toast 警告
            toast.warning("请填写必填项");
            return;
        }

        try {
            // 2. 提交 API
            await resourceApi.create({
                name: formData.resourceName,
                kind: ResourceType.S3,
                config: JSON.stringify({
                    endpoint: formData.endpoint,
                    region: formData.region,
                    bucket: formData.bucket,
                    access_key: formData.accessKey,
                    secret_key: formData.secretKey,
                }),
            });

            // 3. 成功逻辑：提示并跳转
            toast.success("资源创建成功");
            navigate("/resources"); // 这里替换成你实际要跳转的路由路径

        } catch (error: any) {
            // 4. 失败逻辑：捕获错误并弹出 Toast
            console.error(error);
            toast.error("创建失败", {
                description: error.message || "服务器发生错误，请稍后再试",
                // 可选：添加一个重试按钮
                action: {
                    label: "重试",
                    onClick: () => handleSubmit(),
                },
            });
        }
    };

    // 辅助函数：生成样式统一的 Input (也可以替换为 Antd 的 <Input />)
    const renderInput = (
        key: keyof S3ConfigState,
        placeholder: string,
        type: string = 'text'
    ) => {
        return (
            <input
                type={type}
                value={formData[key]}
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
                    render: () => renderInput("resourceName", "请输入该资源的唯一标识名称"),
                },
            ],
        },
        {
            groupName: "S3 数据源配置",
            fields: [
                {
                    id: "endpoint",
                    label: "Endpoint (服务地址)",
                    render: () => renderInput("endpoint", "例如: https://s3.amazonaws.com"),
                },
                {
                    id: "region",
                    label: "Region (区域)",
                    render: () => renderInput("region", "例如: us-east-1"),
                },
                {
                    id: "bucket",
                    label: "Bucket (存储桶)",
                    render: () => renderInput("bucket", "请输入 Bucket 名称"),
                },
                {
                    id: "accessKey",
                    label: "Access Key ID",
                    render: () => renderInput("accessKey", "请输入 Access Key"),
                },
                {
                    id: "secretKey",
                    label: "Secret Access Key",
                    render: () => renderInput("secretKey", "请输入 Secret Key", "password"), // 密码类型
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

export default S3ConfigPage;