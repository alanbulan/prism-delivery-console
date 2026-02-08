/**
 * Modal - 通用模态弹窗容器组件
 *
 * 职责：
 * - 提供半透明遮罩层 + 居中玻璃卡片容器
 * - 点击遮罩关闭弹窗
 * - 可被所有页面复用
 */

import type { ReactNode } from "react";

interface ModalProps {
  /** 弹窗内容 */
  children: ReactNode;
  /** 关闭弹窗回调 */
  onClose: () => void;
}

export function Modal({ children, onClose }: ModalProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* 半透明遮罩 */}
      <div
        className="absolute inset-0 bg-black/20 backdrop-blur-sm"
        onClick={onClose}
      />
      {/* 弹窗内容 */}
      <div className="glass-panel relative z-10 w-full max-w-md p-6 shadow-xl">
        {children}
      </div>
    </div>
  );
}
