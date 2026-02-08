/**
 * ModuleCard - 单个业务模块卡片组件
 *
 * 职责：
 * - 展示单个模块的名称和勾选状态
 * - 点击卡片或勾选框均可切换选中状态
 * - 使用 glass-subtle 样式保持 Liquid Glass 风格一致性
 * - 纯展示组件，通过 props 接收数据和回调
 *
 * 需求: 3.2 (可勾选模块列表), 3.3 (点击切换选中状态), 2.3 (卡片形式展示)
 */

import { Package } from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox";

interface ModuleCardProps {
  /** 模块名称 */
  name: string;
  /** 是否选中 */
  checked: boolean;
  /** 切换选中状态的回调 */
  onToggle: () => void;
}

export function ModuleCard({ name, checked, onToggle }: ModuleCardProps) {
  return (
    <div
      role="button"
      tabIndex={0}
      className={`glass-subtle flex cursor-pointer items-center gap-3 px-3 py-2.5 transition-all select-none hover:bg-white/40 ${
        checked ? "ring-1 ring-primary/30 bg-white/35" : ""
      }`}
      onClick={onToggle}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onToggle();
        }
      }}
    >
      {/* 勾选框 - 阻止事件冒泡，避免与卡片点击冲突 */}
      <Checkbox
        checked={checked}
        onCheckedChange={() => onToggle()}
        onClick={(e) => e.stopPropagation()}
        aria-label={`选择模块 ${name}`}
      />

      {/* 模块图标 */}
      <Package className="h-4 w-4 shrink-0 text-blue-500/70" />

      {/* 模块名称 */}
      <span className="truncate text-sm font-medium text-foreground">
        {name}
      </span>
    </div>
  );
}
