"use client";

import * as React from "react";
import { type ColumnDef } from "@tanstack/react-table";
import {
  ArrowDown,
  ArrowRight,
  ArrowUp,
  CheckCircle2,
  Circle,
  HelpCircle,
  Timer,
  XCircle,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { DataTable, type DataTableFilterField } from "@/components/DataTable"; // 假设你的 DataTable 在这里

// --- 1. 类型定义 ---
export type Task = {
  id: string;
  code: string;
  title: string;
  status: "todo" | "in-progress" | "done" | "canceled";
  label: "bug" | "feature" | "documentation";
  priority: "low" | "medium" | "high";
};

// --- 2. 模拟数据 ---
const data: Task[] = [
  {
    id: "TASK-8782",
    code: "TASK-8782",
    title: "你无法压缩 bin/ 目录下的文件",
    status: "in-progress",
    label: "bug",
    priority: "high",
  },
  {
    id: "TASK-7878",
    code: "TASK-7878",
    title: "我们需要重写整个鉴权系统",
    status: "todo",
    label: "feature",
    priority: "medium",
  },
  {
    id: "TASK-7839",
    code: "TASK-7839",
    title: "更新文档中的 API 参考部分",
    status: "done",
    label: "documentation",
    priority: "low",
  },
  {
    id: "TASK-5562",
    code: "TASK-5562",
    title: "添加黑暗模式支持",
    status: "todo",
    label: "feature",
    priority: "high",
  },
  {
    id: "TASK-8686",
    code: "TASK-8686",
    title: "修复登录页面的对齐问题",
    status: "canceled",
    label: "bug",
    priority: "low",
  },
];

// --- 3. 辅助配置（用于图标和标签映射） ---
const statuses = [
  { value: "todo", label: "待办", icon: Circle },
  { value: "in-progress", label: "进行中", icon: Timer },
  { value: "done", label: "已完成", icon: CheckCircle2 },
  { value: "canceled", label: "已取消", icon: XCircle },
];

const priorities = [
  { value: "low", label: "低", icon: ArrowDown },
  { value: "medium", label: "中", icon: ArrowRight },
  { value: "high", label: "高", icon: ArrowUp },
];

// --- 4. 列定义 ---
export const columns: ColumnDef<Task>[] = [
  {
    id: "select",
    header: ({ table }) => (
      <Checkbox
        checked={
          table.getIsAllPageRowsSelected() ||
          (table.getIsSomePageRowsSelected() && "indeterminate")
        }
        onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
        aria-label="Select all"
        className="translate-y-[2px]"
      />
    ),
    cell: ({ row }) => (
      <Checkbox
        checked={row.getIsSelected()}
        onCheckedChange={(value) => row.toggleSelected(!!value)}
        aria-label="Select row"
        className="translate-y-[2px]"
      />
    ),
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "code",
    header: "任务编号",
    cell: ({ row }) => <div className="w-[80px]">{row.getValue("code")}</div>,
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "title",
    header: "标题",
    cell: ({ row }) => {
      const label = row.original.label;
      return (
        <div className="flex space-x-2">
          {label && <Badge variant="outline">{label}</Badge>}
          <span className="max-w-[500px] truncate font-medium">
            {row.getValue("title")}
          </span>
        </div>
      );
    },
  },
  {
    accessorKey: "status",
    header: "状态",
    cell: ({ row }) => {
      const status = statuses.find(
        (status) => status.value === row.getValue("status"),
      );

      if (!status) return null;

      return (
        <div className="flex w-[100px] items-center">
          {status.icon && (
            <status.icon className="mr-2 h-4 w-4 text-muted-foreground" />
          )}
          <span>{status.label}</span>
        </div>
      );
    },
    // 🔥 关键点：自定义筛选逻辑
    // 如果不加这个，TanStack Table 默认是精准匹配，多选会导致找不到数据
    filterFn: (row, id, value) => {
      return value.includes(row.getValue(id));
    },
  },
  {
    accessorKey: "priority",
    header: "优先级",
    cell: ({ row }) => {
      const priority = priorities.find(
        (priority) => priority.value === row.getValue("priority"),
      );

      if (!priority) return null;

      return (
        <div className="flex items-center">
          {priority.icon && (
            <priority.icon className="mr-2 h-4 w-4 text-muted-foreground" />
          )}
          <span>{priority.label}</span>
        </div>
      );
    },
    // 🔥 关键点：同上，必须包含此逻辑
    filterFn: (row, id, value) => {
      return value.includes(row.getValue(id));
    },
  },
];

// --- 5. 页面组件 ---
export default function TaskPage() {
  // 定义需要显示的筛选器
  const filterFields: DataTableFilterField<Task>[] = [
    {
      label: "状态",
      value: "status",
      options: statuses,
    },
    {
      label: "优先级",
      value: "priority",
      options: priorities,
    },
  ];

  return (
    <div className="hidden h-full flex-1 flex-col space-y-8 p-8 md:flex">
      <div className="flex items-center justify-between space-y-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">欢迎回来!</h2>
          <p className="text-muted-foreground">这里是你本月的任务清单概览。</p>
        </div>
      </div>

      <DataTable
        data={data}
        columns={columns}
        searchKey="title" // 告诉表格搜索哪个字段
        filters={filterFields} // 传入刚才定义的筛选配置
      />
    </div>
  );
}
