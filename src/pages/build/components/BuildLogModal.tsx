/**
 * BuildLogModal - 构建日志模态框
 *
 * 职责：
 * - 以终端风格展示构建过程的实时日志
 * - 自动滚动到最新日志行
 * - 构建完成后可手动关闭
 */

import { useEffect, useRef } from "react";
import { X, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Modal } from "@/components/ui/modal";

interface BuildLogModalProps {
  /** 日志行列表 */
  logs: string[];
  /** 是否正在构建中 */
  isBuilding: boolean;
  /** 关闭模态框回调 */
  onClose: () => void;
}

export function BuildLogModal({ logs, isBuilding, onClose }: BuildLogModalProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  // 新日志到达时自动滚动到底部
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <Modal onClose={isBuilding ? () => {} : onClose} maxWidthClass="max-w-lg">
      <div className="flex flex-col gap-3">
        {/* 标题栏 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {isBuilding && <Loader2 className="h-4 w-4 animate-spin text-primary" />}
            <h3 className="text-sm font-semibold text-foreground">
              {isBuilding ? "构建中..." : "构建日志"}
            </h3>
          </div>
          {!isBuilding && (
            <Button variant="ghost" size="xs" onClick={onClose}>
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>

        {/* 日志内容区域（终端风格） */}
        <div
          ref={scrollRef}
          className="rounded-md bg-zinc-900 p-3 font-mono text-xs leading-relaxed text-zinc-300 overflow-auto"
          style={{ maxHeight: "360px", minHeight: "200px" }}
        >
          {logs.length === 0 ? (
            <span className="text-zinc-500">等待日志...</span>
          ) : (
            logs.map((line, i) => (
              <div key={i} className="whitespace-pre-wrap break-all">
                {line}
              </div>
            ))
          )}
        </div>
      </div>
    </Modal>
  );
}
