/**
 * ChangelogModal - 版本更新记录弹窗组件
 *
 * 职责：
 * - 展示版本更新历史记录
 * - 独立维护更新数据，方便后续版本迭代时修改
 * - 通过 Modal 容器弹出展示
 */

import { History } from "lucide-react";
import { Modal } from "@/components/ui/modal";

/** 更新记录条目 */
export interface ChangelogEntry {
  version: string;
  date: string;
  changes: string[];
}

/** 版本更新历史（最新在前，新版本在此数组头部添加即可） */
export const CHANGELOG: ChangelogEntry[] = [
  {
    version: "0.4.0",
    date: "2026-02-09",
    changes: [
      "构建系统重构：排除式骨架拷贝 + BFS 传递依赖分析",
      "新增敏感文件自动排除（.env 等）",
      "算法优化：时间戳后缀、增量哈希缓存、Rayon 并行哈希、ZIP 流式写入",
      "新增签名索引 + AI 项目报告生成命令注册",
      "分析页按钮样式统一为 outline",
      "修复 index_project_signatures / generate_project_report 未注册的运行时 Bug",
    ],
  },
  {
    version: "0.3.0",
    date: "2026-02-08",
    changes: [
      "新增项目分析模块（文件索引 + 依赖拓扑 + 语义搜索 + AI 报告）",
      "新增 D3.js 力导向图 / 树形依赖拓扑可视化",
      "新增 Embedding 向量搜索（SQLite BLOB + 余弦相似度）",
      "新增静态签名提取 + LLM 项目报告生成",
      "构建系统增强：实时日志推送 + 可配置 modules 目录",
      "新增 import 路径重写器（FastAPI / Vue3）",
      "新增构建日志弹窗 + 构建历史管理（删除/清理）",
      "新增客户模块配置持久化 + 版本号自动递增",
      "设置页增强：LLM/Embedding 模型配置 + 自动索引",
      "侧边栏导航重构 + 废弃页面清理",
    ],
  },
  {
    version: "0.2.0",
    date: "2026-02-08",
    changes: [
      "新增多项目、多技术栈管理（V2 架构）",
      "新增 Vue3 项目扫描与构建支持",
      "后端重构为分层架构（commands → services → models）",
      "统一错误处理（AppError + thiserror）",
      "集成 Tauri v2 自动更新插件",
      "新增 GitHub Actions CI/CD 自动发布",
      "构建链风险点修复（时间戳命名、模块校验等）",
    ],
  },
  {
    version: "0.1.0",
    date: "2026-02-07",
    changes: [
      "初始版本：FastAPI 项目交付包构建",
      "项目验证、模块扫描、ZIP 打包",
      "自定义无边框窗口 + Liquid Glass 风格",
    ],
  },
];

interface ChangelogModalProps {
  onClose: () => void;
}

export function ChangelogModal({ onClose }: ChangelogModalProps) {
  return (
    <Modal onClose={onClose}>
      {/* 标题 */}
      <div className="mb-4 flex items-center gap-2">
        <History className="h-4 w-4 text-amber-500/70" />
        <h3 className="text-sm font-semibold text-foreground">更新记录</h3>
      </div>

      {/* 更新列表（可滚动） */}
      <div className="flex max-h-80 flex-col gap-4 overflow-y-auto pr-1">
        {CHANGELOG.map((entry) => (
          <div key={entry.version} className="flex flex-col gap-1.5">
            <div className="flex items-center gap-2">
              <span className="rounded-full bg-accent px-2 py-0.5 text-xs font-medium text-accent-foreground">
                v{entry.version}
              </span>
              <span className="text-xs text-muted-foreground">{entry.date}</span>
            </div>
            <ul className="ml-1 flex flex-col gap-0.5">
              {entry.changes.map((change, i) => (
                <li key={i} className="flex items-start gap-1.5 text-xs text-muted-foreground">
                  <span className="mt-1.5 h-1 w-1 shrink-0 rounded-full bg-muted-foreground/40" />
                  {change}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </Modal>
  );
}
