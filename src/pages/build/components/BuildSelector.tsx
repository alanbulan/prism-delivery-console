/**
 * BuildSelector - 项目和客户选择器组件
 *
 * 职责：
 * - 项目下拉选择器
 * - 客户标签选择器（点击选中、悬停显示删除）
 * - 内联添加客户表单
 */

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { ChevronDown, Plus, X } from "lucide-react";
import type { Project, Client } from "@/types";

interface BuildSelectorProps {
  projects: Project[];
  selectedProjectId: number | null;
  clients: Client[];
  selectedClientId: number | null;
  onProjectChange: (id: number | null) => void;
  onClientChange: (id: number | null) => void;
  /** 客户创建成功后的回调，用于刷新客户列表 */
  onClientCreated: () => void;
}

export function BuildSelector({
  projects,
  selectedProjectId,
  clients,
  selectedClientId,
  onProjectChange,
  onClientChange,
  onClientCreated,
}: BuildSelectorProps) {
  const [showAddClient, setShowAddClient] = useState(false);
  const [newClientName, setNewClientName] = useState("");
  const [addingClient, setAddingClient] = useState(false);
  // 客户重命名状态
  const [editingClientId, setEditingClientId] = useState<number | null>(null);
  const [editingName, setEditingName] = useState("");

  /** 添加客户并关联到当前项目 */
  const handleAddClient = async () => {
    const trimmed = newClientName.trim();
    if (!trimmed) {
      toast.error("客户名称不能为空");
      return;
    }
    if (!selectedProjectId) return;

    setAddingClient(true);
    try {
      const created = await invoke<Client>("db_create_client", {
        name: trimmed,
        projectIds: [selectedProjectId],
      });
      toast.success(`客户「${created.name}」已创建`);
      setNewClientName("");
      setShowAddClient(false);
      onClientCreated();
      // 自动选中新创建的客户
      onClientChange(created.id);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setAddingClient(false);
    }
  };

  /** 删除客户 */
  const handleDeleteClient = async (clientId: number, clientName: string) => {
    try {
      await invoke("db_delete_client", { id: clientId });
      toast.success(`客户「${clientName}」已删除`);
      // 如果删除的是当前选中的客户，清空选择
      if (selectedClientId === clientId) {
        onClientChange(null);
      }
      onClientCreated(); // 刷新客户列表
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 开始编辑客户名称（双击触发） */
  const handleStartEdit = (client: Client) => {
    setEditingClientId(client.id);
    setEditingName(client.name);
  };

  /** 提交客户重命名 */
  const handleRenameClient = async () => {
    if (!editingClientId) return;
    const trimmed = editingName.trim();
    if (!trimmed) {
      toast.error("客户名称不能为空");
      return;
    }
    try {
      await invoke("db_update_client", { id: editingClientId, name: trimmed });
      toast.success("客户名称已更新");
      setEditingClientId(null);
      setEditingName("");
      onClientCreated(); // 刷新客户列表
    } catch (err) {
      toast.error(String(err));
    }
  };

  /** 取消编辑 */
  const handleCancelEdit = () => {
    setEditingClientId(null);
    setEditingName("");
  };

  return (
    <section className="glass flex flex-col gap-4 p-4">
      {/* 项目 + 客户并排布局 */}
      <div className="flex flex-wrap items-start gap-4">
        {/* 左侧：项目选择器 */}
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

        {/* 右侧：客户标签选择器 */}
        {selectedProjectId && (
          <div className="flex min-w-48 flex-1 flex-col gap-1.5">
            <div className="flex items-center justify-between">
              <label className="text-xs font-medium text-muted-foreground">
                客户
              </label>
              <button
                type="button"
                onClick={() => setShowAddClient(!showAddClient)}
                className="flex items-center gap-0.5 rounded px-1.5 py-0.5 text-xs text-primary hover:bg-accent"
                title="添加客户"
              >
                <Plus className="h-3 w-3" />
                <span>添加</span>
              </button>
            </div>

            {/* 添加客户内联表单 */}
            {showAddClient && (
              <div className="flex gap-1.5">
                <input
                  type="text"
                  value={newClientName}
                  onChange={(e) => setNewClientName(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleAddClient()}
                  placeholder="输入客户名称"
                  className="flex-1 rounded-lg border border-border bg-background px-2.5 py-1.5 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
                  autoFocus
                />
                <button
                  type="button"
                  onClick={handleAddClient}
                  disabled={addingClient}
                  className="shrink-0 rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50"
                >
                  {addingClient ? "..." : "确定"}
                </button>
              </div>
            )}

            {/* 客户标签列表 */}
            {clients.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {clients.map((c) => {
                  const isSelected = selectedClientId === c.id;
                  const isEditing = editingClientId === c.id;

                  // 编辑模式：显示内联输入框
                  if (isEditing) {
                    return (
                      <div key={c.id} className="flex items-center gap-1">
                        <input
                          type="text"
                          value={editingName}
                          onChange={(e) => setEditingName(e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") handleRenameClient();
                            if (e.key === "Escape") handleCancelEdit();
                          }}
                          onBlur={handleRenameClient}
                          className="w-24 rounded-lg border border-primary bg-background px-2 py-1 text-sm text-foreground outline-none focus:ring-2 focus:ring-ring"
                          autoFocus
                        />
                      </div>
                    );
                  }

                  return (
                    <div
                      key={c.id}
                      className={`group flex items-center gap-1 rounded-lg border px-3 py-1.5 text-sm transition-colors ${
                        isSelected
                          ? "border-primary bg-primary text-primary-foreground"
                          : "border-border bg-background text-foreground hover:border-primary/50 hover:bg-accent"
                      }`}
                    >
                      <button
                        type="button"
                        onClick={() => onClientChange(isSelected ? null : c.id)}
                        onDoubleClick={() => handleStartEdit(c)}
                        className="text-left"
                        title="单击选择，双击重命名"
                      >
                        {c.name}
                      </button>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteClient(c.id, c.name);
                        }}
                        className={`ml-0.5 shrink-0 rounded-full p-0.5 opacity-0 transition-opacity group-hover:opacity-100 ${
                          isSelected
                            ? "hover:bg-primary-foreground/20"
                            : "hover:bg-destructive/10 hover:text-destructive"
                        }`}
                        title={`删除客户「${c.name}」`}
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </div>
                  );
                })}
              </div>
            ) : (
              !showAddClient && (
                <p className="text-sm text-muted-foreground">
                  暂无客户，请点击"添加"
                </p>
              )
            )}
          </div>
        )}
      </div>
    </section>
  );
}
