/**
 * 拓扑可视化组件 (TopologyView)
 *
 * 职责：
 * - 使用 D3.js 渲染力导向图 / 树形图
 * - 支持文件级 / 目录级粒度切换
 * - 全屏展开模式
 * - 节点搜索高亮、点击详情、依赖数量缩放
 * - 孤立节点可选隐藏
 */

import { useEffect, useRef, useState, useCallback, useMemo } from "react";
import * as d3 from "d3";
import {
  GitBranch, FolderTree, FileCode, Folder,
  Maximize2, Minimize2, EyeOff, Eye, Search, X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import type { DependencyGraph } from "@/types";

/** 视图模式 */
type ViewMode = "force" | "tree";
/** 粒度模式 */
type GranularityMode = "file" | "directory";

interface TopologyViewProps {
  graph: DependencyGraph;
}

/** D3 节点类型 */
interface GraphNode extends d3.SimulationNodeDatum {
  id: string;
  /** 节点所属目录（用于着色） */
  group: string;
  /** 依赖数量（出度 + 入度，用于节点大小缩放） */
  degree: number;
}

/** D3 边类型 */
interface GraphLink extends d3.SimulationLinkDatum<GraphNode> {
  source: string | GraphNode;
  target: string | GraphNode;
}

/** 将文件级图数据聚合为目录级 */
function aggregateToDirectory(graph: DependencyGraph): DependencyGraph {
  const getDir = (path: string) => {
    const idx = path.lastIndexOf("/");
    return idx >= 0 ? path.substring(0, idx) : "(root)";
  };

  const dirSet = new Set<string>();
  graph.nodes.forEach((n) => dirSet.add(getDir(n)));
  const nodes = Array.from(dirSet);

  // 聚合边：去重
  const edgeSet = new Set<string>();
  const edges: DependencyGraph["edges"] = [];
  for (const e of graph.edges) {
    const srcDir = getDir(e.source);
    const tgtDir = getDir(e.target);
    if (srcDir === tgtDir) continue;
    const key = `${srcDir}->${tgtDir}`;
    if (!edgeSet.has(key)) {
      edgeSet.add(key);
      edges.push({ source: srcDir, target: tgtDir });
    }
  }

  return { nodes, edges };
}

/** 过滤掉孤立节点（无任何依赖边的节点） */
function filterIsolatedNodes(graph: DependencyGraph): DependencyGraph {
  const connected = new Set<string>();
  graph.edges.forEach((e) => {
    connected.add(e.source);
    connected.add(e.target);
  });
  return {
    nodes: graph.nodes.filter((n) => connected.has(n)),
    edges: graph.edges,
  };
}

export function TopologyView({ graph }: TopologyViewProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("force");
  const [granularity, setGranularity] = useState<GranularityMode>("file");
  /** 全屏展开状态 */
  const [expanded, setExpanded] = useState(false);
  /** 是否隐藏孤立节点 */
  const [hideIsolated, setHideIsolated] = useState(true);
  /** 搜索关键词 */
  const [searchTerm, setSearchTerm] = useState("");
  /** 当前选中的节点（点击查看详情） */
  const [selectedNode, setSelectedNode] = useState<string | null>(null);

  /** 获取当前粒度下的图数据（含孤立节点过滤） */
  const getGraphData = useCallback(() => {
    const base = granularity === "directory" ? aggregateToDirectory(graph) : graph;
    return hideIsolated ? filterIsolatedNodes(base) : base;
  }, [graph, granularity, hideIsolated]);

  /** 计算选中节点的依赖详情 */
  const nodeDetail = useMemo(() => {
    if (!selectedNode) return null;
    const data = getGraphData();
    const dependsOn = data.edges.filter((e) => e.source === selectedNode).map((e) => e.target);
    const dependedBy = data.edges.filter((e) => e.target === selectedNode).map((e) => e.source);
    return { id: selectedNode, dependsOn, dependedBy };
  }, [selectedNode, getGraphData]);

  /** Esc 退出全屏 */
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && expanded) setExpanded(false);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [expanded]);

  /** 渲染力导向图 */
  const renderForceGraph = useCallback(() => {
    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const container = svgRef.current?.parentElement;
    if (!container) return;
    const width = container.clientWidth;
    const height = container.clientHeight;

    svg.attr("width", width).attr("height", height);

    const data = getGraphData();
    if (data.nodes.length === 0) return;

    // 计算每个节点的度数（出度 + 入度）
    const degreeMap = new Map<string, number>();
    data.nodes.forEach((n) => degreeMap.set(n, 0));
    data.edges.forEach((e) => {
      degreeMap.set(e.source, (degreeMap.get(e.source) || 0) + 1);
      degreeMap.set(e.target, (degreeMap.get(e.target) || 0) + 1);
    });

    // 构建节点和边数据
    const nodes: GraphNode[] = data.nodes.map((id) => ({
      id,
      group: id.lastIndexOf("/") >= 0 ? id.substring(0, id.lastIndexOf("/")) : "(root)",
      degree: degreeMap.get(id) || 0,
    }));
    const nodeMap = new Map(nodes.map((n) => [n.id, n]));
    const links: GraphLink[] = data.edges
      .filter((e) => nodeMap.has(e.source) && nodeMap.has(e.target))
      .map((e) => ({ source: e.source, target: e.target }));

    // 颜色比例尺（按目录分组着色）
    const groups = Array.from(new Set(nodes.map((n) => n.group)));
    const color = d3.scaleOrdinal(d3.schemeTableau10).domain(groups);

    // 节点大小比例尺（依赖越多越大）
    const maxDegree = Math.max(...nodes.map((n) => n.degree), 1);
    const radiusScale = d3.scaleSqrt().domain([0, maxDegree]).range([4, 16]);

    // 缩放行为
    const g = svg.append("g");
    svg.call(
      d3.zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.1, 4])
        .on("zoom", (event) => g.attr("transform", event.transform)) as any
    );

    // 箭头标记
    g.append("defs")
      .append("marker")
      .attr("id", "arrowhead")
      .attr("viewBox", "0 -5 10 10")
      .attr("refX", 20)
      .attr("refY", 0)
      .attr("markerWidth", 6)
      .attr("markerHeight", 6)
      .attr("orient", "auto")
      .append("path")
      .attr("d", "M0,-5L10,0L0,5")
      .attr("fill", "#94a3b8");

    // 力模拟 — 增大间距，减少密集
    const simulation = d3
      .forceSimulation<GraphNode>(nodes)
      .force("link", d3.forceLink<GraphNode, GraphLink>(links).id((d) => d.id).distance(120))
      .force("charge", d3.forceManyBody().strength(-400))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collision", d3.forceCollide((d: GraphNode) => radiusScale(d.degree) + 8));

    // 绘制边
    const link = g
      .append("g")
      .selectAll("line")
      .data(links)
      .join("line")
      .attr("stroke", "#cbd5e1")
      .attr("stroke-width", 1)
      .attr("marker-end", "url(#arrowhead)");

    // 绘制节点
    const node = g
      .append("g")
      .selectAll("circle")
      .data(nodes)
      .join("circle")
      .attr("r", (d) => radiusScale(d.degree))
      .attr("fill", (d) => color(d.group))
      .attr("stroke", "#fff")
      .attr("stroke-width", 1.5)
      .attr("cursor", "pointer")
      .on("click", (_event, d) => {
        setSelectedNode((prev) => (prev === d.id ? null : d.id));
      })
      .call(
        d3.drag<SVGCircleElement, GraphNode>()
          .on("start", (event, d) => {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
          })
          .on("drag", (event, d) => {
            d.fx = event.x;
            d.fy = event.y;
          })
          .on("end", (event, d) => {
            if (!event.active) simulation.alphaTarget(0);
            d.fx = null;
            d.fy = null;
          }) as any
      );

    // 节点标签 — 默认隐藏，hover 时显示
    const label = g
      .append("g")
      .selectAll("text")
      .data(nodes)
      .join("text")
      .text((d) => {
        const idx = d.id.lastIndexOf("/");
        return idx >= 0 ? d.id.substring(idx + 1) : d.id;
      })
      .attr("font-size", 9)
      .attr("dx", (d) => radiusScale(d.degree) + 4)
      .attr("dy", 3)
      .attr("fill", "#64748b")
      .attr("opacity", 0)
      .attr("pointer-events", "none");

    // hover 显示标签
    node
      .on("mouseenter", (_event, d) => {
        label.attr("opacity", (l) => (l.id === d.id ? 1 : 0));
      })
      .on("mouseleave", () => {
        label.attr("opacity", 0);
      });

    // Tooltip（完整路径）
    node.append("title").text((d) => `${d.id}\n依赖数: ${d.degree}`);

    // 搜索高亮
    const lowerSearch = searchTerm.toLowerCase();
    if (lowerSearch) {
      node.attr("opacity", (d) => (d.id.toLowerCase().includes(lowerSearch) ? 1 : 0.15));
      link.attr("opacity", 0.08);
      // 匹配的节点始终显示标签
      label.attr("opacity", (d) => (d.id.toLowerCase().includes(lowerSearch) ? 1 : 0));
    }

    // 更新位置
    simulation.on("tick", () => {
      link
        .attr("x1", (d) => (d.source as GraphNode).x!)
        .attr("y1", (d) => (d.source as GraphNode).y!)
        .attr("x2", (d) => (d.target as GraphNode).x!)
        .attr("y2", (d) => (d.target as GraphNode).y!);
      node.attr("cx", (d) => d.x!).attr("cy", (d) => d.y!);
      label.attr("x", (d) => d.x!).attr("y", (d) => d.y!);
    });

    return () => { simulation.stop(); };
  }, [getGraphData, searchTerm]);

  /** 渲染树形图 — 动态高度，避免标签重叠 */
  const renderTreeGraph = useCallback(() => {
    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const container = svgRef.current?.parentElement;
    if (!container) return;
    const width = container.clientWidth;
    const containerHeight = container.clientHeight;

    const data = getGraphData();
    if (data.nodes.length === 0) return;

    // 构建层级数据
    const inDegree = new Map<string, number>();
    data.nodes.forEach((n) => inDegree.set(n, 0));
    data.edges.forEach((e) => {
      inDegree.set(e.target, (inDegree.get(e.target) || 0) + 1);
    });

    const children = new Map<string, string[]>();
    data.edges.forEach((e) => {
      if (!children.has(e.source)) children.set(e.source, []);
      children.get(e.source)!.push(e.target);
    });

    // 多根节点支持：所有入度为 0 的节点都作为根
    const roots = data.nodes.filter((n) => (inDegree.get(n) || 0) === 0);

    interface TreeNode { name: string; children?: TreeNode[]; }

    const visited = new Set<string>();
    const buildTree = (id: string): TreeNode => {
      visited.add(id);
      const kids = (children.get(id) || []).filter((c) => !visited.has(c));
      return {
        name: id,
        children: kids.length > 0 ? kids.map((c) => buildTree(c)) : undefined,
      };
    };

    // 构建所有根的子树，而非只取第一个根
    const rootTrees = (roots.length > 0 ? roots : [data.nodes[0]]).map((r) => buildTree(r));
    const unvisited = data.nodes.filter((n) => !visited.has(n));
    const fullTree: TreeNode = {
      name: "(project)",
      children: [...rootTrees, ...unvisited.map((n) => ({ name: n }))],
    };

    const root = d3.hierarchy(fullTree);
    const leafCount = root.leaves().length;

    // 动态计算树高度：每个叶子节点至少 22px 间距，保证标签不重叠
    const minRowHeight = 22;
    const dynamicHeight = Math.max(containerHeight, leafCount * minRowHeight + 40);

    // SVG 尺寸设为动态高度，容器可滚动
    svg.attr("width", width).attr("height", dynamicHeight);

    const g = svg.append("g").attr("transform", "translate(40, 20)");
    svg.call(
      d3.zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.1, 4])
        .on("zoom", (event) => g.attr("transform", event.transform)) as any
    );

    const treeLayout = d3.tree<TreeNode>().size([dynamicHeight - 40, width - 200]);
    treeLayout(root);

    g.selectAll("path.link")
      .data(root.links())
      .join("path")
      .attr("class", "link")
      .attr("fill", "none")
      .attr("stroke", "#cbd5e1")
      .attr("stroke-width", 1)
      .attr(
        "d",
        d3
          .linkHorizontal<d3.HierarchyLink<TreeNode>, d3.HierarchyPointNode<TreeNode>>()
          .x((d) => d.y!)
          .y((d) => d.x!) as any
      );

    const nodeG = g
      .selectAll("g.node")
      .data(root.descendants())
      .join("g")
      .attr("class", "node")
      .attr("transform", (d) => `translate(${d.y},${d.x})`);

    nodeG
      .append("circle")
      .attr("r", 4)
      .attr("fill", (d) => (d.children ? "#6366f1" : "#22c55e"))
      .attr("stroke", "#fff")
      .attr("stroke-width", 1);

    nodeG
      .append("text")
      .attr("dx", 8)
      .attr("dy", 3)
      .attr("font-size", 9)
      .attr("fill", "#64748b")
      .text((d) => {
        const name = d.data.name;
        const idx = name.lastIndexOf("/");
        return idx >= 0 ? name.substring(idx + 1) : name;
      });

    nodeG.append("title").text((d) => d.data.name);
  }, [getGraphData]);

  // 根据视图模式渲染
  useEffect(() => {
    if (viewMode === "force") {
      const cleanup = renderForceGraph();
      return cleanup;
    } else {
      renderTreeGraph();
    }
  }, [viewMode, renderForceGraph, renderTreeGraph]);

  // 窗口大小变化 / 全屏切换时重新渲染
  useEffect(() => {
    const handleResize = () => {
      if (viewMode === "force") renderForceGraph();
      else renderTreeGraph();
    };
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [viewMode, renderForceGraph, renderTreeGraph]);

  // 全屏切换后延迟重新渲染（等 DOM 更新）
  useEffect(() => {
    const timer = setTimeout(() => {
      if (viewMode === "force") renderForceGraph();
      else renderTreeGraph();
    }, 50);
    return () => clearTimeout(timer);
  }, [expanded]);

  const graphData = getGraphData();

  return (
    <section
      className={
        expanded
          ? "fixed inset-0 z-50 flex flex-col bg-background"
          : "glass flex flex-1 flex-col gap-0 overflow-hidden"
      }
    >
      {/* 工具栏 */}
      <div className="flex items-center justify-between border-b border-border px-4 py-2">
        <div className="flex items-center gap-2">
          {/* 全屏切换 */}
          <Button
            variant="ghost"
            size="xs"
            onClick={() => setExpanded((v) => !v)}
            title={expanded ? "退出全屏 (Esc)" : "全屏展开"}
          >
            {expanded ? <Minimize2 className="h-4 w-4" /> : <Maximize2 className="h-4 w-4" />}
          </Button>
          <span className="text-sm font-medium text-foreground">依赖拓扑</span>
        </div>

        <div className="flex items-center gap-2">
          {/* 节点搜索 */}
          <div className="relative">
            <Search className="absolute left-2 top-1/2 h-3 w-3 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="搜索节点..."
              className="h-7 w-36 rounded-md border border-border bg-background pl-7 pr-6 text-xs text-foreground outline-none focus:ring-1 focus:ring-ring"
            />
            {searchTerm && (
              <button
                onClick={() => setSearchTerm("")}
                className="absolute right-1.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                <X className="h-3 w-3" />
              </button>
            )}
          </div>

          {/* 孤立节点切换 */}
          <Button
            variant="ghost"
            size="xs"
            onClick={() => setHideIsolated((v) => !v)}
            title={hideIsolated ? "显示孤立节点" : "隐藏孤立节点"}
            className="gap-1"
          >
            {hideIsolated ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
            {hideIsolated ? "已隐藏孤立" : "显示全部"}
          </Button>

          {/* 视图切换 */}
          <div className="flex rounded-md border border-border">
            <Button
              variant={viewMode === "force" ? "default" : "ghost"}
              size="xs"
              onClick={() => setViewMode("force")}
              className="gap-1 rounded-r-none"
            >
              <GitBranch className="h-3 w-3" />
              力导向
            </Button>
            <Button
              variant={viewMode === "tree" ? "default" : "ghost"}
              size="xs"
              onClick={() => setViewMode("tree")}
              className="gap-1 rounded-l-none"
            >
              <FolderTree className="h-3 w-3" />
              树形
            </Button>
          </div>

          {/* 粒度切换 */}
          <div className="flex rounded-md border border-border">
            <Button
              variant={granularity === "file" ? "default" : "ghost"}
              size="xs"
              onClick={() => setGranularity("file")}
              className="gap-1 rounded-r-none"
            >
              <FileCode className="h-3 w-3" />
              文件级
            </Button>
            <Button
              variant={granularity === "directory" ? "default" : "ghost"}
              size="xs"
              onClick={() => setGranularity("directory")}
              className="gap-1 rounded-l-none"
            >
              <Folder className="h-3 w-3" />
              目录级
            </Button>
          </div>
        </div>
      </div>

      {/* SVG 画布 + 详情面板 — 树形图时允许滚动 */}
      {/* SVG 画布 — 自适应填满父容器 */}
      <div className="relative w-full flex-1 overflow-auto bg-background/50">
        <svg ref={svgRef} className="w-full" />

        {/* 节点详情面板（点击节点后显示） */}
        {nodeDetail && (
          <div className="absolute right-3 top-3 w-64 rounded-lg border border-border bg-background/95 p-3 shadow-lg backdrop-blur-sm" style={{ position: "sticky", top: 12, float: "right", marginRight: 12 }}>
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs font-medium text-foreground truncate" title={nodeDetail.id}>
                {nodeDetail.id.split("/").pop()}
              </span>
              <button onClick={() => setSelectedNode(null)} className="text-muted-foreground hover:text-foreground">
                <X className="h-3.5 w-3.5" />
              </button>
            </div>
            <div className="text-[10px] text-muted-foreground mb-2 truncate" title={nodeDetail.id}>
              {nodeDetail.id}
            </div>

            {nodeDetail.dependsOn.length > 0 && (
              <div className="mb-2">
                <div className="text-[10px] font-medium text-muted-foreground mb-1">
                  依赖 ({nodeDetail.dependsOn.length})
                </div>
                <div className="max-h-24 overflow-auto space-y-0.5">
                  {nodeDetail.dependsOn.map((t) => (
                    <div key={t} className="text-[10px] text-foreground truncate" title={t}>
                      {t.split("/").pop()}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {nodeDetail.dependedBy.length > 0 && (
              <div>
                <div className="text-[10px] font-medium text-muted-foreground mb-1">
                  被依赖 ({nodeDetail.dependedBy.length})
                </div>
                <div className="max-h-24 overflow-auto space-y-0.5">
                  {nodeDetail.dependedBy.map((s) => (
                    <div key={s} className="text-[10px] text-foreground truncate" title={s}>
                      {s.split("/").pop()}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {nodeDetail.dependsOn.length === 0 && nodeDetail.dependedBy.length === 0 && (
              <div className="text-[10px] text-muted-foreground">无依赖关系</div>
            )}
          </div>
        )}
      </div>

      {/* 底部统计 */}
      <div className="flex items-center justify-between border-t border-border px-4 py-1.5 text-xs text-muted-foreground">
        <span>{graphData.nodes.length} 个节点</span>
        <span>{graphData.edges.length} 条依赖</span>
      </div>
    </section>
  );
}
