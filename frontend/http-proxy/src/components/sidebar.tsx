import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Link, useLocation } from "react-router-dom";

interface SidebarNavProps extends React.HTMLAttributes<HTMLElement> {
  items: {
    href: string;
    title: string;
    icon?: React.ReactNode;
  }[];
}

export function SidebarNav({ className, items, ...props }: SidebarNavProps) {
  const location = useLocation();
  const pathname = location.pathname;

  return (
    <nav
      className={cn(
        "flex space-x-2 lg:flex-col lg:space-x-0 lg:space-y-1",
        className,
      )}
      {...props}
    >
      {items.map((item) => {
        // 简单的全等判断，如果需要处理子路径高亮，可以使用 pathname.startsWith(item.href)
        const isActive = pathname === item.href;

        return (
          <Button
            key={item.href}
            variant="ghost"
            asChild
            className={cn(
              "justify-start hover:bg-muted/50 h-9 px-3",
              isActive
                ? "bg-muted font-semibold hover:bg-muted text-foreground border-l-2 border-primary rounded-none lg:rounded-md lg:border-l-0"
                : "text-muted-foreground font-normal hover:text-foreground",
            )}
          >
            <Link to={item.href} className="flex items-center">
              {item.icon && <span className="mr-2 h-4 w-4">{item.icon}</span>}
              {item.title}
            </Link>
          </Button>
        );
      })}
    </nav>
  );
}
