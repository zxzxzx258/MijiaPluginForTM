# 术语表

## DMS 插件

由 DankMaterialShell（DMS）动态加载的 QML 组件。该项目提供一个 daemon
surface 和一个 DankBar widget surface。

## 指标源

提供一个可展示数值及其状态的外部数据适配器。网络吞吐和米家功率都属于指标源。

## 网络采样

由 DMS 的 `DgopService` 采样并计算的下载和上传速率。新 widget 复用该服务，
因此可直接替换 DMS 内置的 `network_speed_monitor`。

## 米家本地协议

设备在局域网 UDP 54321 端口提供的 miIO 请求协议；请求载荷使用设备 token
派生的 AES-128-CBC 密钥和 IV 加密。

## 米家配置

仅当前用户可读的 `$XDG_CONFIG_HOME/DankMaterialShell/mijia-network-power.json`
（默认 `~/.config/DankMaterialShell/mijia-network-power.json`）。token 不存入
DMS 的普通插件设置文件，也不进入日志或命令行参数。

## _Avoid_

- “内置网速项与本插件并存”：替换时应移除 `network_speed_monitor`，以免重复显示。
- “通用米家功率”：不同型号的 SIID/PIID 不同，功率属性必须由每台设备明确指定。
