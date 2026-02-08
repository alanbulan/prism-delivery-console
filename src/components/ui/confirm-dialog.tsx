/**
 * ConfirmDeleteModal - 通用删除确认弹窗组件
 *
 * 职责：
 * - 展示删除确认提示信息
 * - 提供"取消"和"删除"操作按钮
 * - 可被所有需要删除确认的页面复用
 */

import { X } from "lucide-react";
import { Modal } from "@/components/ui/modal";

interface ConfirmDeleteProps {
  /** 提示文本 */
  message: string;
  /** 确认删除回调 */
  onConfirm: () => void;
  /** 关闭弹窗回调 */
  onClose: () => void;
}

export function ConfirmDeleteModal({ message, onConfirm, onClose }: ConfirmDeleteProps) {
  return (
    <Modal onClose={onClose}>
      <div className="mb-4 flex items-center justify-between">
        <h3 className="text-lg font-semibold text-foreground">确认删除</h3>
        <button
          type="button"
          onClick={onClose}
          className="rounded-md p-1 text-muted-foreground hover:bg-accent"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
      <p className="mb-6 text-sm text-muted-foreground">{message}</p>
      <div className="flex justify-end gap-2">
        <button
          type="button"
          onClick={onClose}
          className="rounded-lg px-4 py-2 text-sm text-muted-foreground hover:bg-accent"
        >
          取消
        </button>
        <button
          type="button"
          onClick={onConfirm}
          className="rounded-lg bg-destructive px-4 py-2 text-sm font-medium text-white hover:opacity-90"
        >
          删除
        </button>
      </div>
    </Modal>
  );
}
