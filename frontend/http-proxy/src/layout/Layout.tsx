import React from "react";
import { Link, Outlet, useLocation } from "react-router-dom";
import {
  LayoutDashboard,
  Cloud,
  ShieldCheck,
  Database,
  Settings,
  ChevronRight,
  Search,
  Box,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Checkbox } from "@/components/ui/checkbox";

const menuItems = [
  { name: "控制台主页", icon: LayoutDashboard, path: "/" },
  { name: "计算资源 (EC2)", icon: Cloud, path: "/ec2" },
  { name: "存储服务 (S3)", icon: Box, path: "/s3" },
  { name: "数据库 (RDS)", icon: Database, path: "/rds" },
  { name: "安全与身份 (IAM)", icon: ShieldCheck, path: "/iam" },
];

const AWSLayout = () => {
  const location = useLocation();

  return (
    <div className="flex h-screen w-full bg-[#f2f3f3] text-[#232f3e] overflow-hidden">
      {/* 侧边栏 */}
      <aside className="w-64 flex flex-col bg-white border-r border-gray-200 shadow-sm">
        {/* Logo 区 */}
        <div className="p-4 flex items-center gap-2">
          <div className="w-8 h-8 bg-[#ff9900] rounded flex items-center justify-center">
            <Cloud className="text-white w-5 h-5" />
          </div>
          <span className="font-bold text-lg tracking-tight">Console</span>
        </div>

        {/* 搜索框 (AWS 风格常用) */}
        <div className="px-4 mb-4">
          <div className="relative">
            <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="查找服务"
              className="pl-8 bg-[#fafafa] border-gray-300 focus-visible:ring-[#ff9900]"
            />
          </div>
        </div>

        <Separator className="opacity-50" />

        {/* 导航区 */}
        <ScrollArea className="flex-1 px-3 py-4">
          <div className="space-y-1">
            <p className="px-2 mb-2 text-[11px] font-semibold text-gray-500 uppercase tracking-wider">
              常用服务
            </p>
            {menuItems.map((item) => {
              const isActive = location.pathname === item.path;
              return (
                <Link key={item.path} to={item.path}>
                  <Button
                    variant="ghost"
                    className={cn(
                      "w-full justify-start gap-3 px-2 py-1.5 h-9 font-normal transition-colors",
                      isActive
                        ? "bg-[#f1faff] text-[#0071ad] hover:bg-[#f1faff]"
                        : "hover:bg-gray-100 text-[#444]",
                    )}
                  >
                    <item.icon
                      className={cn(
                        "w-4 h-4",
                        isActive ? "text-[#0071ad]" : "text-gray-500",
                      )}
                    />
                    <span className="flex-1 text-left text-sm">
                      {item.name}
                    </span>
                    {isActive && (
                      <div className="w-1 h-4 bg-[#0071ad] rounded-full" />
                    )}
                  </Button>
                </Link>
              );
            })}
          </div>

          <div className="mt-8 space-y-1">
            <p className="px-2 mb-2 text-[11px] font-semibold text-gray-500 uppercase tracking-wider">
              管理
            </p>
            <Button
              variant="ghost"
              className="w-full justify-start gap-3 px-2 h-9 font-normal text-gray-600 hover:bg-gray-100"
            >
              <Settings className="w-4 h-4" />
              <span className="text-sm">设置与首选项</span>
            </Button>
          </div>
        </ScrollArea>

        {/* 侧边栏底部 */}
        <div className="p-4 border-t border-gray-200 bg-gray-50/50">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-full bg-slate-200 flex items-center justify-center text-[10px] font-bold">
              ADMIN
            </div>
            <div className="flex-1 overflow-hidden">
              <p className="text-xs font-semibold truncate">Root-User</p>
              <p className="text-[10px] text-gray-500 truncate">
                Account: 1234-5678
              </p>
            </div>
          </div>
        </div>
      </aside>

      {/* 主体内容区 */}
      <main className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {/* 内容溢出容器 */}
        <ScrollArea className="flex-1">
          <div className="p-8 max-w-7xl mx-auto w-full">
            {/* 页面标题占位 */}
            <div className="mb-6">
              <nav className="flex text-sm text-gray-500 gap-2 items-center mb-2">
                <span>服务</span>
                <ChevronRight className="w-3 h-3" />
                <span className="text-gray-900 font-medium">概览</span>
              </nav>
            </div>

            {/* 真正渲染子路由的地方 */}
            <div className="animate-in fade-in duration-500">
              <Outlet />
            </div>
          </div>
        </ScrollArea>
      </main>
    </div>
  );
};

export default AWSLayout;
