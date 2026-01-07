// columns.tsx
import { type ColumnDef } from "@tanstack/react-table";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { ArrowUpDown, GitPullRequest, Info, CircleDot } from "lucide-react";
import { Button } from "@/components/ui/button";

export type Issue = {
  id: string;
  title: string;
  status: "open" | "closed";
  label: string;
  author: string;
  createdAt: string;
};

export const columns: ColumnDef<Issue>[] = [
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
        // 这里的 translate-y-[2px] 是为了微调 shadcn checkbox 与文字的视觉对齐
        className="translate-y-[2px] ml-1"
      />
    ),
    cell: ({ row }) => (
      <Checkbox
        checked={row.getIsSelected()}
        onCheckedChange={(value) => row.toggleSelected(!!value)}
        aria-label="Select row"
        className="translate-y-[2px] ml-1"
      />
    ),
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "title",
    header: ({ column }) => {
      return (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          className="-ml-3 h-8 data-[state=open]:bg-accent"
        >
          标题
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      );
    },
    cell: ({ row }) => {
      return (
        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            {row.original.status === "open" ? (
              <CircleDot className="h-4 w-4 text-green-600" />
            ) : (
              <CircleDot className="h-4 w-4 text-purple-600" />
            )}
            <span className="font-semibold text-zinc-900 dark:text-zinc-100 hover:text-blue-600 cursor-pointer">
              {row.getValue("title")}
            </span>
            <Badge
              variant="outline"
              className="text-xs font-normal rounded-full"
            >
              {row.original.label}
            </Badge>
          </div>
          <div className="text-xs text-muted-foreground mt-1">
            #{row.original.id} opened by {row.original.author}
          </div>
        </div>
      );
    },
  },
  {
    accessorKey: "status",
    header: "状态",
    cell: ({ row }) => {
      const status = row.getValue("status") as string;
      return (
        <div className="flex flex-col">
          <Badge
            variant={status === "open" ? "default" : "secondary"}
            className="capitalize"
          >
            {status}
          </Badge>
        </div>
      );
    },
  },
  {
    accessorKey: "createdAt",
    header: "创建时间",
    cell: ({ row }) => {
      return (
        <div className="text-zinc-500 text-sm">{row.getValue("createdAt")}</div>
      );
    },
  },
];
