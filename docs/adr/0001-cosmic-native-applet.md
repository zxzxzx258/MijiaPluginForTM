# ADR 0001: 使用 COSMIC 原生面板小程序

## 状态

已接受

## 背景

TrafficMonitor 的公开插件契约是 Windows DLL 和 Win32 API，不能由 COSMIC 面板直接加载。COSMIC 组件实际扫描 XDG 应用目录中的 `X-CosmicApplet=true` 条目，并以独立进程运行 applet。

## 决策

使用 Rust + libcosmic 实现独立的 COSMIC 面板小程序，保留 TrafficMonitor 的“多指标源、定时刷新、面板文本、点击详情”产品形态，但不复制无关的 Windows 源码。网络吞吐使用 Linux 内核计数器；米家使用独立的 miIO UDP 客户端。

## 后果

用户可以把同一个程序加入 COSMIC 面板，新增指标源不需要修改面板本身。米家令牌属于本地敏感配置，程序只从用户配置读取，仓库和日志不保存真实令牌。COSMIC Settings 外部设置页不是当前可靠扩展点，因此首版详情/状态在 applet 弹窗中呈现。
