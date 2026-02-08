/**
 * 关于页面 (AboutPage)
 *
 * 职责：
 * - 展示应用名称、版本号、技术栈信息和团队信息
 * - 版本号通过 Tauri getVersion API 动态读取
 * - 提供"检查更新"功能（Tauri v2 Updater Plugin）
 * - Liquid Glass 风格的精致卡片布局
 *
 * 需求: 11.1, 11.2, 11.3, 11.4, 12.1
 */

import { useEffect, useState, useCallback } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  Layers,
  Users,
  Code2,
  Heart,
  RefreshCw,
  Download,
  CheckCircle2,
  ScrollText,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ChangelogModal } from "@/components/ChangelogModal";

// ============================================================================
// 技术栈图标组件（内联 SVG，避免外部依赖）
// ============================================================================

/** Tauri 图标 */
const TauriIcon = () => (
  <svg viewBox="0 0 24 24" className="h-4 w-4" fill="none">
    <circle cx="8" cy="8" r="3" fill="#FFC131" />
    <circle cx="16" cy="16" r="3" fill="#24C8DB" />
    <path d="M10.5 9.5C12 12 12 12 13.5 14.5" stroke="#333" strokeWidth="1.5" strokeLinecap="round" />
  </svg>
);

/** React 图标 */
const ReactIcon = () => (
  <svg viewBox="0 0 24 24" className="h-4 w-4" fill="none">
    <circle cx="12" cy="12" r="2" fill="#61DAFB" />
    <ellipse cx="12" cy="12" rx="10" ry="4" stroke="#61DAFB" strokeWidth="1" transform="rotate(0 12 12)" />
    <ellipse cx="12" cy="12" rx="10" ry="4" stroke="#61DAFB" strokeWidth="1" transform="rotate(60 12 12)" />
    <ellipse cx="12" cy="12" rx="10" ry="4" stroke="#61DAFB" strokeWidth="1" transform="rotate(120 12 12)" />
  </svg>
);

/** TypeScript 图标 */
const TypeScriptIcon = () => (
  <svg viewBox="0 0 24 24" className="h-4 w-4">
    <rect x="2" y="2" width="20" height="20" rx="3" fill="#3178C6" />
    <text x="12" y="17" textAnchor="middle" fill="white" fontSize="12" fontWeight="bold" fontFamily="sans-serif">TS</text>
  </svg>
);

/** Rust 图标（齿轮简化版） */
const RustIcon = () => (
  <svg viewBox="0 0 24 24" className="h-4 w-4" fill="none">
    <circle cx="12" cy="12" r="8" stroke="#CE422B" strokeWidth="2" />
    <circle cx="12" cy="12" r="3" fill="#CE422B" />
    <line x1="12" y1="2" x2="12" y2="5" stroke="#CE422B" strokeWidth="2" strokeLinecap="round" />
    <line x1="12" y1="19" x2="12" y2="22" stroke="#CE422B" strokeWidth="2" strokeLinecap="round" />
    <line x1="2" y1="12" x2="5" y2="12" stroke="#CE422B" strokeWidth="2" strokeLinecap="round" />
    <line x1="19" y1="12" x2="22" y2="12" stroke="#CE422B" strokeWidth="2" strokeLinecap="round" />
  </svg>
);

/** Zustand 图标（熊掌简化版） */
const ZustandIcon = () => (
  <svg viewBox="0 0 24 24" className="h-4 w-4" fill="none">
    <circle cx="12" cy="13" r="8" fill="#443D3A" />
    <circle cx="9" cy="11" r="1.5" fill="white" />
    <circle cx="15" cy="11" r="1.5" fill="white" />
    <ellipse cx="12" cy="14.5" rx="2.5" ry="1.5" fill="#D4A574" />
  </svg>
);

/** Tailwind CSS 图标 */
const TailwindIcon = () => (
  <svg viewBox="0 0 24 24" className="h-4 w-4" fill="#06B6D4">
    <path d="M12 6C9.33 6 7.67 7.33 7 10c1-1.33 2.17-1.83 3.5-1.5.76.19 1.3.74 1.9 1.35C13.35 10.82 14.5 12 17 12c2.67 0 4.33-1.33 5-4-1 1.33-2.17 1.83-3.5 1.5-.76-.19-1.3-.74-1.9-1.35C15.65 7.18 14.5 6 12 6zM7 12c-2.67 0-4.33 1.33-5 4 1-1.33 2.17-1.83 3.5-1.5.76.19 1.3.74 1.9 1.35C8.35 16.82 9.5 18 12 18c2.67 0 4.33-1.33 5-4-1 1.33-2.17 1.83-3.5 1.5-.76-.19-1.3-.74-1.9-1.35C10.65 13.18 9.5 12 7 12z" />
  </svg>
);

/** 技术栈图标映射 */
const TECH_ICON_MAP: Record<string, React.FC> = {
  Tauri: TauriIcon,
  React: ReactIcon,
  TypeScript: TypeScriptIcon,
  Rust: RustIcon,
  Zustand: ZustandIcon,
  "Tailwind CSS": TailwindIcon,
};

/** 技术栈条目 */
const TECH_STACK = [
  { name: "Tauri", version: "v2", desc: "跨平台桌面框架" },
  { name: "React", version: "19", desc: "UI 渲染引擎" },
  { name: "TypeScript", version: "5.8", desc: "类型安全" },
  { name: "Rust", version: "stable", desc: "后端核心逻辑" },
  { name: "Zustand", version: "5", desc: "状态管理" },
  { name: "Tailwind CSS", version: "4", desc: "原子化样式" },
] as const;

/** 更新状态枚举 */
type UpdateStatus = "idle" | "checking" | "available" | "downloading" | "upToDate" | "error";

export function AboutPage() {
  /** 应用版本号，从 Tauri 配置中读取 */
  const [version, setVersion] = useState<string>("");
  /** 更新状态 */
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>("idle");
  /** 下载进度百分比 */
  const [downloadProgress, setDownloadProgress] = useState(0);
  /** 更新记录弹窗是否可见 */
  const [showChangelog, setShowChangelog] = useState(false);

  useEffect(() => {
    getVersion().then(setVersion).catch(() => setVersion("unknown"));
  }, []);

  /** 检查更新 */
  const handleCheckUpdate = useCallback(async () => {
    try {
      setUpdateStatus("checking");
      const update = await check();

      if (!update) {
        setUpdateStatus("upToDate");
        toast.success("当前已是最新版本");
        // 3 秒后恢复空闲状态
        setTimeout(() => setUpdateStatus("idle"), 3000);
        return;
      }

      // 发现新版本
      setUpdateStatus("available");
      toast.info(`发现新版本 v${update.version}`);

      // 开始下载并安装
      setUpdateStatus("downloading");
      let downloaded = 0;
      let contentLength = 0;

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? 0;
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            if (contentLength > 0) {
              setDownloadProgress(Math.round((downloaded / contentLength) * 100));
            }
            break;
          case "Finished":
            setDownloadProgress(100);
            break;
        }
      });

      toast.success("更新已安装，即将重启应用...");
      // 短暂延迟后重启，让用户看到提示
      setTimeout(() => relaunch(), 1500);
    } catch (err) {
      const msg = String(err);
      // 尚未发布任何 Release 时，updater 会返回此错误，属于正常情况
      if (msg.includes("Could not fetch") || msg.includes("valid release")) {
        setUpdateStatus("upToDate");
        toast.info("当前已是最新版本（暂无已发布的更新）");
      } else {
        setUpdateStatus("error");
        toast.error(`检查更新失败: ${msg}`);
      }
      setTimeout(() => setUpdateStatus("idle"), 3000);
    }
  }, []);

  /** 根据更新状态渲染按钮内容 */
  const renderUpdateButton = () => {
    switch (updateStatus) {
      case "checking":
        return (
          <Button variant="outline" size="sm" disabled className="gap-1.5">
            <RefreshCw className="h-3.5 w-3.5 animate-spin" />
            检查中...
          </Button>
        );
      case "downloading":
        return (
          <Button variant="outline" size="sm" disabled className="gap-1.5">
            <Download className="h-3.5 w-3.5 animate-bounce" />
            下载中 {downloadProgress}%
          </Button>
        );
      case "upToDate":
        return (
          <Button variant="outline" size="sm" disabled className="gap-1.5">
            <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500" />
            已是最新
          </Button>
        );
      default:
        return (
          <Button
            variant="outline"
            size="sm"
            onClick={handleCheckUpdate}
            className="gap-1.5"
          >
            <RefreshCw className="h-3.5 w-3.5" />
            检查更新
          </Button>
        );
    }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* 页面标题栏 */}
      <header className="glass-subtle flex items-center px-5 py-3">
        <h2 className="text-base font-semibold text-foreground">关于</h2>
      </header>

      {/* 主内容区 */}
      <main className="flex flex-1 items-start justify-center overflow-auto p-6">
        <div className="flex w-full max-w-2xl flex-col gap-5">

          {/* 应用标识卡片 */}
          <section className="glass-panel flex flex-col items-center gap-4 p-8">
            {/* 应用图标 - Prism 棱镜概念：多层光学折射效果 */}
            <div className="relative flex h-20 w-20 items-center justify-center">
              {/* 外层光晕 */}
              <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-blue-400/20 via-violet-400/15 to-indigo-400/20 blur-md" />
              {/* 主体容器 */}
              <div className="relative flex h-20 w-20 items-center justify-center rounded-2xl bg-gradient-to-br from-slate-50/90 to-white/80 shadow-lg ring-1 ring-white/60">
                {/* 棱镜三角形 - 使用 CSS 绘制 */}
                <svg viewBox="0 0 48 48" className="h-11 w-11" fill="none">
                  {/* 棱镜主体 */}
                  <defs>
                    <linearGradient id="prism-face-1" x1="0%" y1="0%" x2="100%" y2="100%">
                      <stop offset="0%" stopColor="#6366f1" stopOpacity="0.9" />
                      <stop offset="100%" stopColor="#818cf8" stopOpacity="0.7" />
                    </linearGradient>
                    <linearGradient id="prism-face-2" x1="0%" y1="100%" x2="100%" y2="0%">
                      <stop offset="0%" stopColor="#3b82f6" stopOpacity="0.8" />
                      <stop offset="100%" stopColor="#60a5fa" stopOpacity="0.6" />
                    </linearGradient>
                    <linearGradient id="prism-face-3" x1="100%" y1="0%" x2="0%" y2="100%">
                      <stop offset="0%" stopColor="#8b5cf6" stopOpacity="0.7" />
                      <stop offset="100%" stopColor="#a78bfa" stopOpacity="0.5" />
                    </linearGradient>
                    {/* 彩虹光谱渐变 */}
                    <linearGradient id="spectrum" x1="0%" y1="0%" x2="100%" y2="0%">
                      <stop offset="0%" stopColor="#ef4444" stopOpacity="0.8" />
                      <stop offset="20%" stopColor="#f97316" stopOpacity="0.7" />
                      <stop offset="40%" stopColor="#eab308" stopOpacity="0.7" />
                      <stop offset="60%" stopColor="#22c55e" stopOpacity="0.7" />
                      <stop offset="80%" stopColor="#3b82f6" stopOpacity="0.7" />
                      <stop offset="100%" stopColor="#8b5cf6" stopOpacity="0.8" />
                    </linearGradient>
                  </defs>
                  {/* 入射光线 */}
                  <line x1="2" y1="22" x2="16" y2="22" stroke="#94a3b8" strokeWidth="1.5" strokeLinecap="round" opacity="0.6" />
                  {/* 棱镜三角形 - 三个面 */}
                  <polygon points="24,8 38,36 10,36" fill="url(#prism-face-1)" />
                  <polygon points="24,8 38,36 24,36" fill="url(#prism-face-2)" />
                  <polygon points="24,8 10,36 24,28" fill="url(#prism-face-3)" />
                  {/* 棱镜边框 */}
                  <polygon points="24,8 38,36 10,36" fill="none" stroke="white" strokeWidth="0.8" opacity="0.5" />
                  {/* 折射光谱 - 从棱镜右侧射出的彩虹 */}
                  <line x1="34" y1="18" x2="46" y2="12" stroke="#ef4444" strokeWidth="1.2" strokeLinecap="round" opacity="0.7" />
                  <line x1="35" y1="20" x2="46" y2="16" stroke="#f97316" strokeWidth="1.2" strokeLinecap="round" opacity="0.7" />
                  <line x1="36" y1="22" x2="46" y2="20" stroke="#eab308" strokeWidth="1.2" strokeLinecap="round" opacity="0.7" />
                  <line x1="36" y1="24" x2="46" y2="24" stroke="#22c55e" strokeWidth="1.2" strokeLinecap="round" opacity="0.7" />
                  <line x1="36" y1="26" x2="46" y2="28" stroke="#3b82f6" strokeWidth="1.2" strokeLinecap="round" opacity="0.7" />
                  <line x1="35" y1="28" x2="46" y2="32" stroke="#8b5cf6" strokeWidth="1.2" strokeLinecap="round" opacity="0.7" />
                </svg>
              </div>
            </div>

            <div className="flex flex-col items-center gap-1">
              <h1 className="text-xl font-bold text-foreground">
                Prism Delivery Console
              </h1>
              <p className="text-sm text-muted-foreground">
                多项目交付包构建管理工具
              </p>
            </div>

            {/* 版本标签 + 更新记录按钮 + 检查更新按钮 */}
            <div className="flex items-center gap-3">
              {version && (
                <span className="rounded-full bg-accent px-4 py-1 text-xs font-medium text-accent-foreground">
                  v{version}
                </span>
              )}
              {/* 更新记录触发按钮 */}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowChangelog(true)}
                className="gap-1.5 text-muted-foreground hover:text-foreground"
              >
                <ScrollText className="h-3.5 w-3.5" />
                更新记录
              </Button>
              {renderUpdateButton()}
            </div>
          </section>

          {/* 技术栈卡片 */}
          <section className="glass flex flex-col gap-3 p-5">
            <div className="flex items-center gap-2">
              <Layers className="h-4 w-4 text-blue-500/70" />
              <h3 className="text-sm font-semibold text-foreground">技术栈</h3>
            </div>
            <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
              {TECH_STACK.map((tech) => {
                const Icon = TECH_ICON_MAP[tech.name];
                return (
                  <div
                    key={tech.name}
                    className="glass-subtle flex items-start gap-2.5 px-3 py-2.5"
                  >
                    {/* 技术栈图标 */}
                    {Icon && (
                      <div className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-background/60 ring-1 ring-border/40">
                        <Icon />
                      </div>
                    )}
                    <div className="flex flex-col gap-0.5">
                      <div className="flex items-baseline gap-1.5">
                        <span className="text-sm font-medium text-foreground">
                          {tech.name}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {tech.version}
                        </span>
                      </div>
                      <span className="text-xs text-muted-foreground/70">
                        {tech.desc}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          </section>

          {/* 更新记录卡片 → 已移至 ChangelogModal 弹窗组件 */}

          {/* 项目信息卡片 */}
          <div className="grid grid-cols-2 gap-4">
            {/* 开发团队 */}
            <section className="glass flex flex-col gap-3 p-5">
              <div className="flex items-center gap-2">
                <Users className="h-4 w-4 text-indigo-500/70" />
                <h3 className="text-sm font-semibold text-foreground">
                  开发团队
                </h3>
              </div>
              <p className="text-sm text-muted-foreground">Prism Team</p>
            </section>

            {/* 开源协议 */}
            <section className="glass flex flex-col gap-3 p-5">
              <div className="flex items-center gap-2">
                <Code2 className="h-4 w-4 text-emerald-500/70" />
                <h3 className="text-sm font-semibold text-foreground">
                  项目信息
                </h3>
              </div>
              <p className="text-sm text-muted-foreground">内部工具 · 私有项目</p>
            </section>
          </div>

          {/* 底部致谢 */}
          <div className="flex items-center justify-center gap-1.5 py-2 text-xs text-muted-foreground/60">
            <span>Built with</span>
            <Heart className="h-3 w-3 text-red-400/60" />
            <span>using Tauri + React</span>
          </div>

        </div>
      </main>

      {/* 更新记录弹窗 */}
      {showChangelog && (
        <ChangelogModal onClose={() => setShowChangelog(false)} />
      )}
    </div>
  );
}
