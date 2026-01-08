"use client";

import * as React from "react";
import { type ColumnDef } from "@tanstack/react-table";
import { ResourceType, type Resource } from "@/types/api";
import { resourceApi } from "@/api/resource";
import { DataTable } from "@/components/DataTable";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button"; // 假设你有 Button 组件
import { format } from "date-fns";
import { Copy, Edit, MoreHorizontal, Trash2 } from "lucide-react"; // 引入删除图标
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";

export default function ResourcePage() {
  const [data, setData] = React.useState<Resource[]>([]);
  const [loading, setLoading] = React.useState(true);

  // 1. 定义加载数据的方法，方便在删除后重新刷新列表
  const loadData = async () => {
    setLoading(true);
    try {
      const response = await resourceApi.list({ page: 1, page_size: 10 });
      setData(response);
    } catch (error) {
      console.error("加载资源失败:", error);
    } finally {
      setLoading(false);
    }
  };

  // 2. 删除处理函数
  const onDelete = async (id: string) => {
    if (!confirm("确定要删除该资源吗？")) return;

    try {
      await resourceApi.delete(id);
      // 删除成功后本地过滤掉，或者重新调用 loadData()
      setData((prev) => prev.filter((item) => item.id !== id));
      console.log("删除成功");
    } catch (error) {
      console.error("删除失败:", error);
    }
  };

  const columns: ColumnDef<Resource>[] = [
    {
      accessorKey: "id",
      header: "ID",
      cell: ({ row }) => <span className="font-mono text-xs">{row.original.id}</span>,
    },
    {
      accessorKey: "name",
      header: "资源名称",
    },
    {
      accessorKey: "kind",
      header: "类型",
      cell: ({ row }) => {
        const kind = row.getValue("kind") as ResourceType;
        return (
          <Badge variant={kind === ResourceType.S3 ? "default" : "outline"}>
            {kind}
          </Badge>
        );
      },
    },
    {
      accessorKey: "created_at",
      header: "创建时间",
      cell: ({ row }) => {
        return format(new Date(row.original.created_at), "yyyy-MM-dd HH:mm:ss");
      },
    },
    // --- 新增：操作列 ---
    {
      id: "actions",
      header: "操作",
      cell: ({ row }) => {
        const resource = row.original;

        return (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" className="h-8 w-8 p-0">
                <span className="sr-only">打开菜单</span>
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuLabel>操作项</DropdownMenuLabel>
              <DropdownMenuItem
                onClick={() => navigator.clipboard.writeText(resource.id)}
              >
                <Copy className="mr-2 h-4 w-4" />
                复制 ID
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => console.log("编辑", resource.id)}>
                <Edit className="mr-2 h-4 w-4" />
                编辑资源
              </DropdownMenuItem>
              <DropdownMenuItem
                className="text-red-600 focus:text-red-600 focus:bg-red-50"
                onClick={() => onDelete(resource.id)}
              >
                <Trash2 className="mr-2 h-4 w-4" />
                删除资源
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        );
      },
    },
  ];

  React.useEffect(() => {
    loadData();
  }, []);

  const filterFields = [
    {
      label: "类型",
      value: "kind",
      options: [
        { label: "S3 Storage", value: ResourceType.S3 },
        { label: "Proxy", value: ResourceType.PROXY },
        { label: "SFTP", value: ResourceType.SFTP },
      ],
    },
  ];

  if (loading && data.length === 0) return <div>加载中...</div>;

  return (
    <div className="container mx-auto py-10">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">资源管理</h2>
          <p className="text-muted-foreground">管理并配置您的资源。</p>
        </div>
      </div>
      <DataTable
        columns={columns}
        data={data}
        searchKey="name"
        filters={filterFields}
      />
    </div>
  );
}