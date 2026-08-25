# 米家功率与网速 DMS 插件

面向 [DankMaterialShell](https://github.com/AvengeMedia/DankMaterialShell) 的
DankBar 插件。它以 DMS 原生的网速采样替换 `network_speed_monitor`，并在同一
组件显示局域网米家设备的实时功率、总功率和每台设备状态。

## 功能

- 直接复用 `DgopService.networkRxRate` 与 `networkTxRate`，网速口径和 DMS
  内置组件一致。
- 一个 DMS daemon 并行读取多台设备的 miIO 功率；多个面板实例不会重复请求。
- 点击组件打开详情：查看设备状态、手动刷新、新增、编辑、删除设备，并配置
  型号、功率 `SIID`/`PIID`。
- 面板默认只显示总功率，设备功率和设备管理按设备数量折叠，避免大量设备拉长
  面板；需要时可在设置中开启面板内逐台功率。
- 内置小米 SVG 会按 DMS 当前浅色或深色主题的面板前景色单色显示。
- token 只保存在
  `~/.config/DankMaterialShell/mijia-network-power.json`，helper 拒绝使用
  组或其他用户可读的配置文件。
- 横向和纵向面板均显示总功率；横向面板的逐台功率显示可在 DMS 插件设置中
  手动开启。

## 安装

需要 DMS 1.5.0 或更高版本、Rust 工具链，以及本地网络可访问米家设备。

```fish
git clone --branch DMS https://github.com/zxzxzx258/MijiaPluginForTM.git MijiaPower-MultiDevice-DMS
cd MijiaPower-MultiDevice-DMS
fish install.fish
```

安装脚本只写入当前用户目录：

- `~/.config/DankMaterialShell/plugins/MijiaNetworkPower/`
- `~/.config/DankMaterialShell/mijia-network-power.json`（权限 `0600`）

然后在 DMS 设置的“插件”页启用“米家功率与网速”，在面板项目中添加
`mijiaNetworkPower`。要替换内置网速项，从同一面板移除
`network_speed_monitor`。

## 添加设备

点击面板上的“米家功率与网速”组件，在展开面板中选择“添加设备”。每个设备
需要名称、局域网 IPv4 地址、32 位十六进制 miIO token，以及实际功率属性的
`SIID`/`PIID`。token 输入框默认掩码显示，保存后会以原子写入方式更新受限配置。

新安装会创建没有设备的配置，不需要手动编辑 JSON。`config.example.json` 仅供
自动化或迁移工具使用，不能提交真实 token。

## 从文本迁移

若已有仅包含 IP/token 的本地文本，可用 helper 导入。它只接受相邻的 IP 与 token
记录，候选数量不一致或无法唯一配对时会拒绝写入，避免误将网关地址导入。

```fish
target/release/mijia-power-helper --import-text /path/to/devices.txt --output ~/.config/DankMaterialShell/mijia-network-power.json
```

导入完成后，仍可在组件详情中修改设备名称、型号及 `SIID`/`PIID`。

## 构建与验证

```fish
cargo test
cargo build --release
target/release/mijia-power-helper --config ~/.config/DankMaterialShell/mijia-network-power.json
```

最后一条命令只输出功率读数或错误状态，不会输出 token。单台设备超时、token
无效或属性编号不匹配不会覆盖其他设备的读数。

## 限制

米家固件可能禁用本地 miIO；不同型号的功率属性也可能不是默认的
`SIID=11`、`PIID=2`。出现单设备失败时，在组件详情中针对该设备调整属性编号或
确认其 IP/token 与局域网可达性。
