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
  Github, // 新增图标
  BookOpen, // 新增图标用于文档/wiki感
  GitFork,  // 新增图标
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";

// 保持原有逻辑，但名称和图标可以稍微调整以适配风格（此处保留原名称以便理解映射）
const menuItems = [
  { name: "控制台主页", icon: LayoutDashboard, path: "/" },
  { name: "计算资源 (EC2)", icon: Cloud, path: "/ec2" },
  { name: "存储服务 (S3)", icon: Box, path: "/s3" },
  { name: "数据库 (RDS)", icon: Database, path: "/rds" },
  { name: "安全与身份 (IAM)", icon: ShieldCheck, path: "/iam" },
];

const GitHubLayout = () => {
  const location = useLocation();

  return (
    <div className="flex h-screen w-full bg-white text-[#24292f] overflow-hidden font-sans">
      {/* 侧边栏：GitHub 风格通常侧边栏是浅灰色的 (#f6f8fa) */}
      <aside className="w-72 flex flex-col bg-[#f6f8fa] border-r border-[#d0d7de]">
        
        {/* Logo / 顶部上下文区 */}
        <div className="px-4 py-5 flex items-center gap-3">
          <div className="w-8 h-8 bg-[#24292f] text-white rounded-full flex items-center justify-center">
            <Github className="w-5 h-5" />
          </div>
          <div className="flex flex-col">
            <span className="font-semibold text-sm leading-tight">DevOps-Platform</span>
            <span className="text-xs text-[#57606a]">Enterprise</span>
          </div>
        </div>

        <Separator className="bg-[#d0d7de] mx-4 w-auto mb-2 opacity-60" />

        {/* 导航区 */}
        <ScrollArea className="flex-1 px-3 py-2">
          <div className="space-y-0.5">
            <div className="px-3 py-2 flex items-center justify-between group cursor-pointer hover:bg-[#eaeef2] rounded-md mb-2 transition-colors">
                 <span className="text-xs font-semibold text-[#24292f]">Repositories</span>
                 <BookOpen className="w-3.5 h-3.5 text-[#57606a]" />
            </div>

            {menuItems.map((item) => {
              const isActive = location.pathname === item.path;
              return (
                <Link key={item.path} to={item.path}>
                  <Button
                    variant="ghost"
                    className={cn(
                      "w-full justify-start gap-3 px-3 py-1.5 h-8 font-normal transition-all rounded-md mb-1",
                      isActive
                        ? "bg-[#eaeef2] text-[#24292f] font-semibold shadow-sm border border-[#d0d7de]/50" 
                        : "text-[#57606a] hover:bg-[#eaeef2] hover:text-[#24292f]",
                    )}
                  >
                    <item.icon
                      className={cn(
                        "w-4 h-4",
                        isActive ? "text-[#24292f]" : "text-[#57606a]",
                      )}
                    />
                    <span className="flex-1 text-left text-[14px]">
                      {item.name}
                    </span>
                  </Button>
                </Link>
              );
            })}
          </div>

          <div className="mt-6 space-y-0.5">
            <p className="px-3 mb-2 text-xs font-semibold text-[#57606a]">
              Settings
            </p>
            <Button
              variant="ghost"
              className="w-full justify-start gap-3 px-3 h-8 font-normal text-[#57606a] hover:bg-[#eaeef2] hover:text-[#24292f] rounded-md"
            >
              <Settings className="w-4 h-4" />
              <span className="text-[14px]">Configuration</span>
            </Button>
          </div>
        </ScrollArea>

        {/* 侧边栏底部：用户卡片 */}
        <div className="p-4 border-t border-[#d0d7de] bg-white">
          <div className="flex items-center gap-3 group cursor-pointer">
            <div className="w-8 h-8 rounded-full border border-[#d0d7de] overflow-hidden">
                <img 
                    src="https://github.com/github.png" 
                    alt="User" 
                    className="w-full h-full object-cover"
                />
            </div>
            <div className="flex-1 overflow-hidden">
              <p className="text-sm font-semibold text-[#24292f] group-hover:text-[#0969da] transition-colors">
                @root-user
              </p>
              <p className="text-xs text-[#57606a] truncate flex items-center gap-1">
                 <span className="w-2 h-2 bg-green-500 rounded-full inline-block"></span>
                 Online
              </p>
            </div>
          </div>
        </div>
      </aside>

      {/* 主体内容区 */}
      <main className="flex-1 flex flex-col min-w-0 overflow-hidden bg-white">
        <ScrollArea className="flex-1">
          <div className="p-8 max-w-[1280px] mx-auto w-full">
            {/* 面包屑导航 (GitHub 风格) */}
            <div className="mb-6 border-b border-[#d0d7de] pb-4">
              <nav className="flex text-sm items-center gap-1">
                <span className="text-[#0969da] hover:underline cursor-pointer">root-user</span>
                <span className="text-[#57606a]">/</span>
                <span className="text-[#0969da] hover:underline cursor-pointer font-semibold">devops-console</span>
                <span className="ml-2 px-2 py-0.5 rounded-full border border-[#d0d7de] text-xs text-[#57606a] font-medium">Public</span>
              </nav>
            </div>

            {/* 真正渲染子路由的地方 */}
            <div className="animate-in fade-in duration-300">
               {/* 模拟一个 GitHub 风格的 Tab 栏 */}
                <div className="flex gap-6 mb-6 border-b border-[#d0d7de] text-sm">
                    <div className="pb-2 border-b-2 border-[#fd8c73] font-semibold text-[#24292f] flex items-center gap-2">
                        <Box className="w-4 h-4"/>
                        Overview
                    </div>
                    <div className="pb-2 cursor-pointer text-[#57606a] hover:text-[#24292f] hover:border-b-2 hover:border-[#d0d7de] transition-all flex items-center gap-2">
                        <GitFork className="w-4 h-4"/>
                        Activity
                    </div>
                    <div className="pb-2 cursor-pointer text-[#57606a] hover:text-[#24292f] hover:border-b-2 hover:border-[#d0d7de] transition-all flex items-center gap-2">
                        <Settings className="w-4 h-4"/>
                        Settings
                    </div>
                </div>

              <Outlet />
            </div>
          </div>
        </ScrollArea>
      </main>
    </div>
  );
};

export default GitHubLayout;