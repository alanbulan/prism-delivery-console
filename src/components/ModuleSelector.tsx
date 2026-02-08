/**
 * ModuleSelector - 业务模块选择区域组件
 *
 * 职责：
 * - 从 Zustand store 读取模块列表和选中状态
 * - 提供全选/反选按钮（需求 3.4, 3.5）
 * - 实时显示已选模块计数（需求 3.6）
 * - 以网格布局渲染 ModuleCard 列表（需求 3.2, 2.3）
 * - 无模块时显示空状态提示（需求 2.4）
 *
 * 需求: 3.2, 3.3, 3.4, 3.5, 3.6, 2.3, 2.4
 */

import { CheckSquare, ToggleLeft, PackageOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useAppStore } from "@/store";
import { ModuleCard } from "@/components/ModuleCard";

export function ModuleSelector() {
  const modules = useAppStore((s) => s.modules);
  const selectedModules = useAppStore((s) => s.selectedModules);
  const toggleModule = useAppStore((s) => s.toggleModule);
  const selectAll = useAppStore((s) => s.selectAll);
  const invertSelection = useAppStore((s) => s.invertSelection);
  const projectPath = useAppStore((s) => s.projectPath);

  const totalCount = modules.length;
  const selectedCount = selectedModules.size;

  // 未加载项目或模块列表为空时，显示空状态提示（需求 2.4）
  if (!projectPath || totalCount === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-2">
        <PackageOpen className="h-10 w-10 text-muted-foreground/40" />
        <p className="text-xs text-muted-foreground">
          {!projectPath
            ? "打开项目后显示可选业务模块"
            : "当前项目未扫描到业务模块"}
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col gap-3">
      {/* 工具栏：全选/反选按钮 + 已选计数 */}
      <div className="flex items-center gap-2">
        {/* 全选按钮（需求 3.4） */}
        <Button variant="outline" size="sm" onClick={selectAll}>
          <CheckSquare className="mr-1 h-3.5 w-3.5" />
          全选
        </Button>

        {/* 反选按钮（需求 3.5） */}
        <Button variant="outline" size="sm" onClick={invertSelection}>
          <ToggleLeft className="mr-1 h-3.5 w-3.5" />
          反选
        </Button>

        {/* 已选模块计数（需求 3.6） */}
        <span className="ml-auto text-xs text-muted-foreground">
          已选 {selectedCount}/{totalCount}
        </span>
      </div>

      {/* 模块卡片网格（需求 3.2, 2.3） */}
      <div className="grid grid-cols-2 gap-2 overflow-y-auto">
        {modules.map((mod) => (
          <ModuleCard
            key={mod.name}
            name={mod.name}
            checked={selectedModules.has(mod.name)}
            onToggle={() => toggleModule(mod.name)}
          />
        ))}
      </div>
    </div>
  );
}
