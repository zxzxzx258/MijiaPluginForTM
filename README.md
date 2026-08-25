# 米家功率与网速 DMS 插件

面向 [DankMaterialShell](https://github.com/AvengeMedia/DankMaterialShell) 的
DankBar 复合插件。它复用 DMS 原生网速采样，并通过局域网 miIO 并行读取多台
米家设备的实时功率。

- 当前版本：`0.2.3`
- 作者与维护者：`LinusLIU`
- 运行平台：Linux、DMS 1.5.0 及以上

## 项目家族与分支

这些分支来自同一项目的功能演进，但面向不同桌面和宿主，安装包及配置文件不能
互换：

| 分支 | 版本/阶段 | 平台 | 定位 |
| --- | --- | --- | --- |
| [`main`](https://github.com/zxzxzx258/MijiaPluginForTM/tree/main) | v1.0.0 基线 | Windows / TrafficMonitor | 上游单设备版本，包含最初的 IP/token 获取教程 |
| [`feature/multi-device-power`](https://github.com/zxzxzx258/MijiaPluginForTM/tree/feature/multi-device-power) | 多设备开发线 | Windows / TrafficMonitor | 将每台设备注册为独立显示项，并增加迁移和独立配置 |
| [`cosmic`](https://github.com/zxzxzx258/MijiaPluginForTM/tree/cosmic) | v0.1.0 | Linux / COSMIC | 原生 COSMIC panel applet，加入网速、路由接口选择和多设备功率 |
| [`DMS`](https://github.com/zxzxzx258/MijiaPluginForTM/tree/DMS) | v0.2.3 | Linux / DMS | DMS daemon + DankBar 组件 + 原生设置页，是当前功能最完整的 Linux 分支 |

三个正式平台分支的 README 会互相链接。它们共享 miIO 协议思路和默认功率属性
`SIID=11`、`PIID=2`，但不共享二进制产物，也不会自动互相迁移配置。

## 主要功能

- 复用 `DgopService.networkRxRate` 与 `networkTxRate`，网速以整数显示并自动
  切换 `B/s`、`KB/s`、`MB/s`、`GB/s`；横向组件按内容自然伸缩且不换行。
- 单个 DMS daemon 并行读取所有设备；多个面板实例不会重复访问设备。
- 每台设备独立显示名称、功率和错误状态，单台设备超时不会覆盖其他读数。
- “选择面板显示设备”作为总开关；开启后可在每张设备卡片上逐台勾选。旧配置和
  新增设备默认勾选，取消勾选只隐藏状态栏条目，不停止采样、不删除设备。
- 总功率默认显示在详情汇总区，也可在设置中额外放到状态栏。
- 设备功率和设备管理使用整行可点击的折叠区，设备较多时不会拉成长页面。
- DMS 插件设置页与组件弹层共用完整设备管理：新增、编辑、删除、显示选择、
  IP、token、型号以及功率 `SIID`/`PIID`。
- 内置 Xiaomi SVG 随 DMS 浅色/深色主题按面板前景色单色显示。
- helper 不回显 token，并拒绝读取组或其他用户可读的配置文件。

## 安装与更新

需要 Rust 工具链、DMS 1.5.0 或更高版本，以及本机到米家设备的局域网连通性。

```fish
git clone --branch DMS https://github.com/zxzxzx258/MijiaPluginForTM.git MijiaPower-MultiDevice-DMS
cd MijiaPower-MultiDevice-DMS
fish install.fish
dms restart
```

安装脚本只写入当前用户目录：

- `~/.config/DankMaterialShell/plugins/MijiaNetworkPower/`
- `~/.config/DankMaterialShell/mijia-network-power.json`，权限固定为 `0600`

在 DMS 设置的“插件”页启用“米家功率与网速”，再在 Dank 状态栏的部件列表中
加入 `mijiaNetworkPower`。若它用于替换内置网速项，可从同一栏移除
`network_speed_monitor`。更新后运行 `dms restart` 即可，不需要注销桌面。

## 在 DMS 中配置

打开 DMS 设置 -> 插件，展开“米家功率与网速”。设置页提供：

- 显示网速；
- 显示总功率；
- 选择面板显示设备；
- 功率刷新间隔；
- 完整设备管理。

开启“选择面板显示设备”后，设备卡片上的开关可以操作：勾选设备会出现在状态栏，
取消勾选则仅从状态栏隐藏。关闭总开关时，卡片开关会禁用，设备读数仍可在弹层的
“设备功率”区域查看。

添加设备需要填写：

- 名称：仅用于界面显示；
- IP：设备当前的局域网 IPv4 地址；
- miIO token：32 位十六进制字符串；
- 型号：默认 `local.mijia.device`；
- 功率属性：默认 `SIID=11`、`PIID=2`。

token 输入框默认掩码显示，保存时使用原子写入。同一套编辑器也可从状态栏组件的
弹层中打开，不需要手写 JSON。

## 获取设备 IP 和 token

以下方法整理自本仓库 [`main` 分支的原始教程](https://github.com/zxzxzx258/MijiaPluginForTM/tree/main)。
第三方提取工具会接触米家账户凭据，使用前应检查项目源码、Release 来源和风险；
不要把账户密码或 token 发到聊天、Issue、终端日志或 Git 仓库。

### 方法一：Xiaomi Cloud Tokens Extractor

1. 打开 [Xiaomi Cloud Tokens Extractor Releases](https://github.com/PiotrMachowski/Xiaomi-cloud-tokens-extractor/releases)。
2. 下载与当前系统匹配的官方 Release，并核对下载来源。
3. 运行工具，按提示登录米家账户并选择设备实际所属区域；中国大陆账户通常选择
   China，区域选错可能返回空设备列表。
4. 在结果中找到目标插座，记录该设备的局域网 IP 和 32 位十六进制 token。
5. 回到 DMS 插件设置页添加设备并保存。

Windows Release 通常可以直接运行；具体文件名以项目当前 Release 为准：

```powershell
.\xiaomi_cloud_tokens_extractor.exe
```

### 方法二：python-miio

在隔离的 Python 环境中安装并运行 `python-miio`：

```fish
python -m pip install python-miio
python -m miio.extract_tokens
```

该工具同样需要选择正确区域。命令和登录流程可能随上游版本变化，应以
[`python-miio`](https://github.com/rytilahti/python-miio) 当前文档为准。

### 确认 IP

云端记录的 IP 可能过期。若设备在线但插件无法连接，可在路由器的 DHCP 客户端列表
中按设备名称或 MAC 地址确认当前 IP，并为设备设置 DHCP 地址保留，避免重启路由器后
IP 改变。DMS 主机和设备必须位于可互通的局域网，UDP `54321` 不能被访客网络隔离。

### 凭据检查

- token 必须恰好为 32 个十六进制字符，不带空格、引号或 `0x` 前缀。
- token 不正确通常表现为设备可达但请求解密或响应校验失败。
- 重置设备、重新绑定账号或部分固件升级可能使旧 token 失效，需要重新提取。
- 不要在 README、截图、日志、提交或 Issue 中粘贴真实 IP/token 组合。

## 从本地文本迁移

若已有仅包含 IP/token 的本地文本，可用 helper 导入。它只接受相邻的 IP 与 token
记录；候选数量不一致或无法唯一配对时会拒绝写入，避免误将网关地址导入。

```fish
cargo build --release
target/release/mijia-power-helper --import-text /path/to/devices.txt --output ~/.config/DankMaterialShell/mijia-network-power.json
chmod 600 ~/.config/DankMaterialShell/mijia-network-power.json
dms restart
```

导入后可在 DMS 设置页修改设备名称、显示选择、型号及 `SIID`/`PIID`。

## 构建与验证

```fish
cargo fmt -- --check
cargo test
cargo build --release
target/release/mijia-power-helper --config ~/.config/DankMaterialShell/mijia-network-power.json
```

helper 的探测输出只包含设备 ID、名称、状态栏选择、功率和错误状态，不包含 IP 或
token。当前测试覆盖旧配置迁移、逐设备显示选择、miIO 加密封包和文本导入配对。

运行时可使用脱敏 IPC 检查状态：

```fish
dms ipc call plugins status mijiaNetworkPower
dms ipc call mijiaNetworkPower status
```

正常情况下插件状态为 `loaded`，并且 `configuredDevices` 与 `readableDevices` 等于
实际可用设备数量。

## 版本迭代

- `0.2.1`：建立 DMS daemon、DankBar 网速/功率组件、设备编辑和主题单色 LOGO。
- `0.2.2`：修复配置 URL 路径导致的功率空白；网速改为整数自动单位和动态宽度；
  设备清单改为紧凑折叠布局。
- `0.2.3`：增加 DMS 原生设置页设备管理、可靠的整行折叠点击、逐设备状态栏选择、
  默认分开显示设备功率，并将作者统一为 `LinusLIU`。

## 限制与排查

米家固件可能禁用本地 miIO，不同型号的功率属性也可能不是默认值。出现问题时按
以下顺序检查：

1. 插件是否为 `loaded`；
2. 设备 IP 是否仍是路由器中的当前地址；
3. token 是否为当前设备的 32 位 token；
4. 主机与设备之间的 UDP `54321` 是否可达；
5. 设备实际功率属性是否为 `SIID=11`、`PIID=2`。

单台设备失败不会阻止其他设备更新。修改 IP/token 或属性编号后可在弹层中点击
“刷新功率”，无需注销。
