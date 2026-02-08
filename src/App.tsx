/**
 * 根组件 - Prism Delivery Console V2
 *
 * 简化为渲染 AppShell 顶层布局组件。
 * 所有布局逻辑（标题栏、导航、页面路由、背景装饰）
 * 已迁移至 AppShell 中统一管理。
 */

import { AppShell } from "@/AppShell";

function App() {
  return <AppShell />;
}

export default App;
