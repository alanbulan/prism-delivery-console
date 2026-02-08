/**
 * 项目概览面板 (OverviewPanel)
 *
 * 职责：三段式布局
 * 1. 基础统计（技术栈、文件数、语言分布）
 * 2. 数据准备状态面板（扫描/签名/Embedding 状态 + 操作按钮）
 * 3. AI 报告区域（模式选择 + 生成按钮 + Markdown 渲染）
 */

import { useState } from "react";
import {
  Loader2, BarChart3, Code2, FolderOpen, Layers, FileCode,
  ScanSearch, Fingerprint, Brain, Zap, Microscope, CheckCircle2, Circle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import Markdown from "react-markdown";
import type { ProjectOverview } from "@/types";

interface OverviewPanelProps {
  /** 项目概览数据 */
  overview: ProjectOverview | null;
  /** 是否正在加载概览 */
  loading: boolean;
  /** 是否已选择项目 */
  hasProject: boolean;
  /** 加载概览回调 */
  onLoad: () => void;
  /** 是否已扫描文件索引 */
  hasFileIndex: boolean;
  /** 是否正在扫描 */
  scanning: boolean;
  /** 扫描回调 */
  onScan: () => void;
  /** 是否正在提取签名 */
  indexingSignatures: boolean;
  /** 签名是否已索引 */
  signaturesIndexed: boolean;
  /** 提取签名回调 */
  onIndexSignatures: () => void;
  /** 是否正在生成 Embedding */
  embeddingAll: boolean;
  /** 生成 Embedding 回调 */
  onEmbedAll: () => void;
  /** AI 报告内容 */
  report: string | null;
  /** 是否正在生成报告 */
  generatingReport: boolean;
  /** 生成报告回调 */
  onGenerateReport: (mode: "fast" | "deep") => void;
}

export function OverviewPanel({
  overview, loading, hasProject, onLoad,
  hasFileIndex, scanning, onScan,
  indexingSignatures, signaturesIndexed, onIndexSignatures,
  embeddingAll, onEmbedAll,
  report, generatingReport, onGenerateReport,
}: OverviewPanelProps) {
  const [reportMode, setReportMode] = useState<"fast" | "deep">("fast");

  // 未选择项目
  if (!hasProject) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <BarChart3 className="h-10 w-10 opacity-30" />
          <p className="text-sm">请先选择项目</p>
        </div>
      </div>
    );
  }

  // 未加载概览
  if (!overview) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="flex flex-col items-center gap-3 text-muted-foreground">
          <BarChart3 className="h-10 w-10 opacity-30" />
          <p className="text-sm">点击下方按钮加载项目概览</p>
          <Button onClick={onLoad} disabled={loading} className="gap-2">
            {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <BarChart3 className="h-4 w-4" />}
            {loading ? "加载中..." : "加载概览"}
          </Button>
        </div>
      </div>
    );
  }

  const maxLines = Math.max(...overview.languages.map((l) => l.line_count), 1);

  return (
    <div className="flex flex-col gap-5">
      {/* ========== 第一部分：基础统计 ========== */}
      <section>
        {/* 统计卡片 */}
        <div className="grid grid-cols-3 gap-3 mb-4">
          <StatCard icon={<FileCode className="h-5 w-5 text-blue-500" />} label="文件数" value={overview.total_files} />
          <StatCard icon={<Code2 className="h-5 w-5 text-green-500" />} label="代码行数" value={overview.total_lines.toLocaleString()} />
          <StatCard icon={<FolderOpen className="h-5 w-5 text-amber-500" />} label="目录数" value={overview.total_dirs} />
        </div>

        {/* 技术栈标签 */}
        {overview.tech_stack.length > 0 && (
          <div className="glass p-4 mb-3">
            <div className="flex items-center gap-2 mb-3 text-sm font-medium text-foreground">
              <Layers className="h-4 w-4" />
              技术栈
            </div>
            <div className="flex flex-wrap gap-2">
              {overview.tech_stack.map((tag) => (
                <span key={tag} className="rounded-full bg-primary/10 px-3 py-1 text-xs font-medium text-primary">
                  {tag}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* 语言分布 */}
        {overview.languages.length > 0 && (
          <div className="glass p-4 mb-3">
            <div className="flex items-center gap-2 mb-3 text-sm font-medium text-foreground">
              <BarChart3 className="h-4 w-4" />
              语言分布
            </div>
            <div className="flex flex-col gap-2">
              {overview.languages.map((lang) => (
                <div key={lang.language} className="flex items-center gap-3">
                  <span className="w-32 shrink-0 text-xs text-foreground truncate">{lang.language}</span>
                  <div className="flex-1 h-4 rounded-full bg-muted overflow-hidden">
                    <div
                      className="h-full rounded-full bg-primary/60 transition-all"
                      style={{ width: `${(lang.line_count / maxLines) * 100}%` }}
                    />
                  </div>
                  <span className="w-20 shrink-0 text-right text-xs text-muted-foreground">
                    {lang.file_count} 文件
                  </span>
                  <span className="w-24 shrink-0 text-right text-xs text-muted-foreground">
                    {lang.line_count.toLocaleString()} 行
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 入口文件 */}
        {overview.entry_files.length > 0 && (
          <div className="glass p-4">
            <div className="flex items-center gap-2 mb-3 text-sm font-medium text-foreground">
              <FileCode className="h-4 w-4" />
              入口文件
            </div>
            <div className="flex flex-wrap gap-2">
              {overview.entry_files.map((f) => (
                <span key={f} className="rounded-md border border-border bg-muted/50 px-2 py-1 text-xs text-foreground">
                  {f}
                </span>
              ))}
            </div>
          </div>
        )}
      </section>

      {/* ========== 第二部分：数据准备状态 ========== */}
      <section className="glass p-4">
        <div className="flex items-center gap-2 mb-4 text-sm font-medium text-foreground">
          <ScanSearch className="h-4 w-4" />
          数据准备
        </div>
        <p className="text-xs text-muted-foreground mb-4">
          生成 AI 报告前，需要先完成以下数据准备步骤。每一步都会为报告提供更丰富的分析素材。
        </p>
        <div className="flex flex-col gap-3">
          {/* 步骤 1：文件扫描 */}
          <PrepStep
            icon={<ScanSearch className="h-4 w-4" />}
            label="文件扫描"
            description="扫描项目文件并建立索引"
            done={hasFileIndex}
            loading={scanning}
            onAction={onScan}
            actionLabel="扫描"
          />
          {/* 步骤 2：签名提取 */}
          <PrepStep
            icon={<Fingerprint className="h-4 w-4" />}
            label="签名提取"
            description="提取代码中的类、函数、接口声明"
            done={signaturesIndexed}
            loading={indexingSignatures}
            onAction={onIndexSignatures}
            actionLabel="提取"
            disabled={!hasFileIndex}
          />
          {/* 步骤 3：Embedding（可选） */}
          <PrepStep
            icon={<Brain className="h-4 w-4" />}
            label="Embedding 向量"
            description="生成文件向量用于语义搜索（可选）"
            done={false}
            loading={embeddingAll}
            onAction={onEmbedAll}
            actionLabel="生成"
            disabled={!hasFileIndex}
            optional
          />
        </div>
      </section>

      {/* ========== 第三部分：AI 报告 ========== */}
      <section className="glass p-4">
        <div className="flex items-center gap-2 mb-4 text-sm font-medium text-foreground">
          <Microscope className="h-4 w-4" />
          AI 项目分析报告
        </div>

        {/* 模式选择 + 生成按钮 */}
        <div className="flex items-center gap-3 mb-4">
          <div className="flex rounded-lg border border-border overflow-hidden">
            <button
              onClick={() => setReportMode("fast")}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs transition-colors ${
                reportMode === "fast"
                  ? "bg-primary text-primary-foreground"
                  : "bg-background text-muted-foreground hover:text-foreground"
              }`}
            >
              <Zap className="h-3 w-3" />
              快速模式
            </button>
            <button
              onClick={() => setReportMode("deep")}
              className={`flex items-center gap-1.5 px-3 py-1.5 text-xs transition-colors ${
                reportMode === "deep"
                  ? "bg-primary text-primary-foreground"
                  : "bg-background text-muted-foreground hover:text-foreground"
              }`}
            >
              <Microscope className="h-3 w-3" />
              深度模式
            </button>
          </div>
          <Button
            size="sm"
            onClick={() => onGenerateReport(reportMode)}
            disabled={generatingReport || !signaturesIndexed}
            className="gap-2"
          >
            {generatingReport ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Brain className="h-3.5 w-3.5" />
            )}
            {generatingReport ? "生成中..." : "生成报告"}
          </Button>
          {!signaturesIndexed && (
            <span className="text-xs text-muted-foreground">
              请先完成文件扫描和签名提取
            </span>
          )}
        </div>
        <p className="text-xs text-muted-foreground mb-3">
          {reportMode === "fast"
            ? "快速模式：收集签名+概览+依赖，1 次 LLM 调用生成报告。适合中小型项目（<100 文件）。"
            : "深度模式：大型项目先压缩签名摘要，再生成报告。适合大型项目（>=100 文件），需 2-3 次 LLM 调用。"}
        </p>

        {/* 报告内容 */}
        {generatingReport && (
          <div className="flex items-center justify-center py-12 text-muted-foreground">
            <Loader2 className="h-6 w-6 animate-spin mr-3" />
            <span className="text-sm">正在生成报告，请稍候...</span>
          </div>
        )}
        {report && !generatingReport && (
          <div className="prose prose-sm dark:prose-invert max-w-none border-t border-border pt-4">
            <Markdown>{report}</Markdown>
          </div>
        )}
        {!report && !generatingReport && (
          <div className="flex items-center justify-center py-8 text-muted-foreground">
            <p className="text-xs">报告将在此处显示</p>
          </div>
        )}
      </section>
    </div>
  );
}

/** 统计卡片子组件 */
function StatCard({ icon, label, value }: { icon: React.ReactNode; label: string; value: string | number }) {
  return (
    <div className="glass flex items-center gap-3 p-4">
      {icon}
      <div>
        <div className="text-lg font-semibold text-foreground">{value}</div>
        <div className="text-xs text-muted-foreground">{label}</div>
      </div>
    </div>
  );
}

/** 数据准备步骤子组件 */
function PrepStep({
  icon, label, description, done, loading, onAction, actionLabel, disabled, optional,
}: {
  icon: React.ReactNode;
  label: string;
  description: string;
  done: boolean;
  loading: boolean;
  onAction: () => void;
  actionLabel: string;
  disabled?: boolean;
  optional?: boolean;
}) {
  return (
    <div className="flex items-center gap-3 rounded-lg border border-border p-3">
      {/* 状态图标 */}
      <div className="shrink-0">
        {done ? (
          <CheckCircle2 className="h-5 w-5 text-green-500" />
        ) : (
          <Circle className="h-5 w-5 text-muted-foreground/40" />
        )}
      </div>
      {/* 图标 + 文字 */}
      <div className="shrink-0 text-muted-foreground">{icon}</div>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium text-foreground">
          {label}
          {optional && <span className="ml-1.5 text-xs text-muted-foreground font-normal">（可选）</span>}
        </div>
        <div className="text-xs text-muted-foreground truncate">{description}</div>
      </div>
      {/* 操作按钮 */}
      <Button
        variant="outline"
        size="sm"
        onClick={onAction}
        disabled={loading || disabled}
        className="gap-1.5 shrink-0"
      >
        {loading ? <Loader2 className="h-3 w-3 animate-spin" /> : null}
        {loading ? "处理中..." : done ? `重新${actionLabel}` : actionLabel}
      </Button>
    </div>
  );
}
