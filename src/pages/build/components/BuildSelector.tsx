/**
 * BuildSelector - 项目和客户选择器组件
 *
 * 职责：
 * - 项目下拉选择器
 * - 客户下拉选择器
 */

import { ChevronDown } from "lucide-react";
import type { Project, Client } from "@/types";

interface BuildSelectorProps {
  projects: Project[];
  selectedProjectId: number | null;
  clients: Client[];
  selectedClientId: number | null;
  onProjectChange: (id: number | null) => void;
  onClientChange: (id: number | null) => void;
}

export function BuildSelector({
  projects,
  selectedProjectId,
  clients,
  selectedClientId,
  onProjectChange,
  onClientChange,
}: BuildSelectorProps) {
  return (
    <section className="glass flex flex-wrap items-end gap-4 p-4">
      {/* 项目选择器 */}
      <div className="flex min-w-48 flex-1 flex-col gap-1.5">
        <label className="text-xs font-medium text-muted-foreground">
          项目
        </label>
        <div className="relative">
          <select
            value={selectedProjectId ?? ""}
            onChange={(e) => {
              const val = e.target.value;
              onProjectChange(val ? Number(val) : null);
            }}
            className="w-full appearance-none rounded-lg border border-border bg-background px-3 py-2 pr-8 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
          >
            <option value="">-- 请选择项目 --</option>
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name} ({p.tech_stack_type})
              </option>
            ))}
          </select>
          <ChevronDown className="pointer-events-none absolute right-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        </div>
      </div>

      {/* 客户选择器 */}
      <div className="flex min-w-48 flex-1 flex-col gap-1.5">
        <label className="text-xs font-medium text-muted-foreground">
          客户
        </label>
        {selectedProjectId && clients.length === 0 ? (
          <p className="px-1 py-2 text-sm text-muted-foreground">
            该项目暂无关联客户
          </p>
        ) : (
          <div className="relative">
            <select
              value={selectedClientId ?? ""}
              onChange={(e) => {
                const val = e.target.value;
                onClientChange(val ? Number(val) : null);
              }}
              disabled={!selectedProjectId || clients.length === 0}
              className="w-full appearance-none rounded-lg border border-border bg-background px-3 py-2 pr-8 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
            >
              <option value="">-- 请选择客户 --</option>
              {clients.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
            <ChevronDown className="pointer-events-none absolute right-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          </div>
        )}
      </div>
    </section>
  );
}
