/**
 * 构建交付页面 - 本地类型定义
 */

/** 构建历史记录行 Props */
export interface BuildRecordRowProps {
  /** 构建时间 */
  createdAt: string;
  /** 客户名称 */
  clientName: string;
  /** 模块数量 */
  moduleCount: number;
  /** 输出路径 */
  outputPath: string;
  /** 打开文件夹回调 */
  onOpenFolder: (path: string) => void;
}
