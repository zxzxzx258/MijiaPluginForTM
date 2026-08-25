# ADR 0002: DMS 插件使用 DgopService 和本地 miIO helper

## 背景

DMS 的内置网速监视器已通过 `DgopService.networkRxRate` 和
`DgopService.networkTxRate` 统一采样。米家本地 miIO 则需要 UDP 54321、
AES-128-CBC 和 token，QML/Quickshell 没有适合直接承担该协议的稳定 API。

## 决策

`mijiaNetworkPower` 使用 DMS 复合插件的两个 surface：

- widget 复用 `DgopService`，因此可替换 `network_speed_monitor` 而不改变
  DMS 的网速计量来源；
- daemon 以固定间隔运行仓库构建的 `mijia-power-helper`，并通过
  `PluginGlobalVar` 向所有 widget 实例发布设备读数；
- token 只位于 `~/.config/DankMaterialShell/mijia-network-power.json`，helper
  拒绝读取组或其他用户可读的配置文件。

## 后果

多显示器或多个面板中的 widget 不会重复发起 miIO 请求。DMS 插件只需复制
QML 和一个本地构建的 helper，不依赖系统级安装或提权。DMS 的网络速率仍会
按其当前实现汇总接口，和被替换的小组件保持一致。
