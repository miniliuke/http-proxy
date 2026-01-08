import React, { useRef, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";

// 扩展类型定义，增加 description 支持，符合 GitHub 表单习惯
export type TaskField = {
  id: string;
  label?: string;
  description?: string; // 新增：字段下方的辅助说明
  render: () => React.ReactNode;
};

export type TaskFieldGroup = {
  groupName: string;
  description?: string; // 新增：分组的简短说明
  fields: TaskField[];
};

export type TaskTemplateFormProps = {
  groups: TaskFieldGroup[];
  onSubmit?: () => void;
  submitText?: string;
  className?: string;
};

export const TaskTemplateForm: React.FC<TaskTemplateFormProps> = ({
  groups,
  onSubmit,
  submitText = "Save changes", // GitHub 风格默认文案
  className,
}) => {
  const [activeGroup, setActiveGroup] = useState<string | null>(
    groups[0]?.groupName || null
  );
  
  // 使用 Ref 引用右侧滚动容器，避免全局 document 查找干扰
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const isScrollingRef = useRef(false); // 防止点击滚动时触发 Observer

  const handleScrollTo = (groupName: string) => {
    isScrollingRef.current = true;
    setActiveGroup(groupName);
    
    const el = document.getElementById(`group-${groupName}`);
    if (el && scrollContainerRef.current) {
      // 计算偏移量，预留 header 高度
      const topOffset = el.offsetTop - 24; 
      scrollContainerRef.current.scrollTo({
        top: topOffset,
        behavior: "smooth",
      });
    }

    // 延时释放锁
    setTimeout(() => {
      isScrollingRef.current = false;
    }, 500);
  };

  useEffect(() => {
    const container = scrollContainerRef.current;
    if (!container) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (isScrollingRef.current) return;

        // 找到当前视口中可见比例最高的元素，或者最靠上的元素
        const visibleEntry = entries.find(e => e.isIntersecting);
        if (visibleEntry) {
          const id = visibleEntry.target.id.replace("group-", "");
          setActiveGroup(id);
        }
      },
      {
        root: container, // 指定滚动容器为 root
        rootMargin: "-10% 0px -60% 0px", // 视口判定区域调整，更符合阅读习惯
        threshold: 0,
      }
    );

    groups.forEach((group) => {
      const el = document.getElementById(`group-${group.groupName}`);
      if (el) observer.observe(el);
    });

    return () => observer.disconnect();
  }, [groups]);

  return (
    <div className={cn("flex w-full h-full bg-background isolate", className)}>
      {/* --- 左侧导航栏 --- */}
      <aside className="w-64 shrink-0 hidden md:block border-r bg-muted/10">
        <div className="sticky top-0 h-full">
          <ScrollArea className="h-full py-6 pr-4 pl-6">
            <nav className="flex flex-col space-y-1">
              {groups.map((group) => {
                const isActive = activeGroup === group.groupName;
                return (
                  <button
                    key={group.groupName}
                    onClick={() => handleScrollTo(group.groupName)}
                    className={cn(
                      "group flex items-center justify-between w-full text-left rounded-md px-3 py-2 text-sm transition-all duration-200",
                      isActive
                        ? "bg-accent text-accent-foreground font-medium shadow-sm ring-1 ring-inset ring-border" 
                        : "text-muted-foreground hover:bg-muted hover:text-foreground"
                    )}
                  >
                    <span>{group.groupName}</span>
                    {isActive && (
                      <div className="w-1 h-1 rounded-full bg-primary ml-2" />
                    )}
                  </button>
                );
              })}
            </nav>
          </ScrollArea>
        </div>
      </aside>

      {/* --- 右侧表单内容 --- */}
      <main 
        ref={scrollContainerRef}
        className="flex-1 overflow-y-auto scroll-smooth"
      >
        <div className="max-w-4xl mx-auto p-6 md:p-10 pb-20">
          <form
            onSubmit={(e) => {
              e.preventDefault();
              onSubmit?.();
            }}
            className="space-y-10"
          >
            {groups.map((group) => (
              <section 
                key={group.groupName} 
                id={`group-${group.groupName}`}
                className="scroll-mt-6" // 锚点定位偏移
              >
                {/* 
                  GitHub 风格：Boxed Group 
                  外层有边框，圆角，Header 与 Body 分离
                */}
                <div className="border rounded-md bg-card shadow-sm overflow-hidden">
                  {/* Group Header */}
                  <div className="bg-muted/40 border-b px-6 py-4">
                    <h3 className="text-base font-semibold leading-none tracking-tight text-foreground">
                      {group.groupName}
                    </h3>
                    {group.description && (
                      <p className="text-sm text-muted-foreground mt-1.5">
                        {group.description}
                      </p>
                    )}
                  </div>

                  {/* Group Content */}
                  <div className="p-6 space-y-6 bg-white dark:bg-zinc-950">
                    {group.fields.map((field, index) => (
                      <div key={field.id} className="group/field">
                        <div className="space-y-2">
                          {field.label && (
                            <label className="text-sm font-semibold leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                              {field.label}
                            </label>
                          )}
                          
                          {/* 渲染具体的输入控件 */}
                          <div className="relative">
                            {field.render()}
                          </div>

                          {field.description && (
                            <p className="text-[13px] text-muted-foreground">
                              {field.description}
                            </p>
                          )}
                        </div>
                        
                        {/* 只要不是最后一个字段，就显示分割线 (可选，视复杂度而定) */}
                        {index < group.fields.length - 1 && (
                          <Separator className="mt-6 opacity-40" />
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              </section>
            ))}

            {/* 底部保存栏 */}
            {onSubmit && (
              <div className="sticky bottom-6 flex justify-end">
                <Button 
                  type="submit" 
                  size="sm"
                  className="bg-[#1f883d] hover:bg-[#1a7f37] text-white shadow-sm font-medium border border-[rgba(27,31,36,0.15)] h-9 px-4"
                >
                  {submitText}
                </Button>
              </div>
            )}
          </form>
        </div>
      </main>
    </div>
  );
};