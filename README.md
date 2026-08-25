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

## 架构与职责

DMS 使用 QML 实现界面和插件生命周期；Rust 只承担本地 miIO 协议 worker，并不
运行第二套桌面界面。这样做是因为 miIO 需要原始 UDP `54321` 通信、hello 握手、
AES-128-CBC/MD5 加密和多设备并发，而 DMS/Quickshell 的 QML 公共接口并不提供
一套稳定的原始 UDP 加密客户端。

| 模块 | 文件 | 负责内容 |
| --- | --- | --- |
| DankBar 组件 | `MijiaNetworkPowerWidget.qml` | 网速格式化、单色 LOGO、状态栏逐设备显示、弹层汇总与手动刷新 |
| DMS daemon | `MijiaNetworkPowerDaemon.qml` | 定时/手动启动 helper、解析脱敏 JSON、发布全局读数、提供状态 IPC |
| 插件设置页 | `MijiaNetworkPowerSettings.qml` | 网速、总功率、逐设备显示选择和刷新间隔设置 |
| 设备编辑器 | `MijiaDeviceEditor.qml` | 原子保存配置、掩码 token 输入、设备增删改和每台设备的状态栏选择 |
| 折叠组件 | `MijiaCollapsibleSection.qml` | 整行点击、阻止 Flickable 抢走点击、展开动画 |
| 协议 helper | `mijia-power-helper` | 配置权限检查、miIO UDP/加密通信、并发读取、脱敏结果输出 |
| 用户配置 | `mijia-network-power.json` | 每台设备的 IP、token、型号、属性编号和 `show_in_bar` 状态 |

### 数据流与刷新流程

```text
DMS 设置页 / 弹层编辑器
        |  原子写入，收紧为 0600
        v
~/.config/DankMaterialShell/mijia-network-power.json
        |
        v
QML daemon --config <配置路径> --> mijia-power-helper
        |                              |
        |                              +--> 对每台已配置设备并发执行 miIO UDP 读取
        |                              +--> 只输出 id/name/show_in_bar/watts/error
        v
pluginService.setGlobalVar("readings")
        |
        v
DankBar QML：网速 + 被勾选设备功率 + 弹层详情
```

1. 插件加载时，DankBar 组件向 `DgopService` 请求网速采样；daemon 先发布空读数，
   再启动 helper。
2. helper 检查配置权限是否为 `0600` 或更严格，读取设备配置，并对每台设备并发
   发送 miIO 请求。单台失败会带自己的错误返回，其他设备继续更新。
3. daemon 解析 helper 的 JSON，将其写入 DMS 的 `readings` 全局变量；组件订阅该
   变量，不直接读取 token 配置文件。
4. 设置页或弹层保存后会发送 `refreshRequest` 全局变量，daemon 立即刷新；状态
   IPC `dms ipc call mijiaNetworkPower status` 只返回设备数量、可读数量、错误和时间。
5. 定时器按“功率刷新间隔”再次执行。刷新期间已有读数保留，避免界面闪空。

`show_in_bar` 是普通显示偏好，不是凭据。它与功率读数一起返回给 QML，用于决定
哪台设备进入状态栏；IP 和 token 永远不会从 helper 返回。

### 安全与网络边界

- helper 只连接配置中出现的局域网 IP，不扫描局域网、不登录米家云、不包含任何
  硬编码的设备 IP 或 token。
- 同一个预编译 helper 对所有用户都是通用程序；每位用户只会读取自己本机的配置，
  并连接自己配置的设备。
- token 仅存在受限用户配置文件中。helper 标准输出、DMS IPC、README 和 CI artifact
  都不包含 token。
- 任何预编译程序都需要信任发布来源。CI 产物附带 SHA-256，源码、工作流和构建命令
  都公开，仍建议从本仓库 Actions 下载或自行 fork 后构建。

## 安装与更新

需要 DMS 1.5.0 或更高版本，以及本机到米家设备的局域网连通性。本地从源码构建
helper 时需要 Rust；使用下方 GitHub Actions 产物时不需要在本机安装 Rust。

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

## 不安装 Rust：使用 GitHub Actions 编译 helper

`DMS` 分支包含 [Build DMS helper workflow](https://github.com/zxzxzx258/MijiaPluginForTM/actions/workflows/build-dms-helper.yml)。
每次修改 helper 源码时会构建两个 Linux x86_64 目标，并将二进制和 `SHA256SUMS`
作为 workflow artifact 上传：

| artifact | 适用范围 |
| --- | --- |
| `mijia-power-helper-x86_64-unknown-linux-musl` | 优先选择；静态 musl 版本通常兼容更多 x86_64 Linux 发行版 |
| `mijia-power-helper-x86_64-unknown-linux-gnu` | 针对使用 glibc 的常规 Linux 环境 |

使用步骤：

1. 在 GitHub 的 Actions 页面打开最新成功的 `Build DMS helper` 运行记录，下载与本机
   匹配的 artifact；没有权限运行上游工作流时，fork 仓库后在自己的 Actions 页面点击
   `Run workflow`。
2. 解压 artifact，在其目录验证校验和：

   ```fish
   sha256sum -c SHA256SUMS
   ```

3. 仍需 clone 本分支以取得 QML 和安装脚本，但用预编译 helper 跳过 Cargo：

   ```fish
   fish install.fish --helper /path/to/mijia-power-helper
   dms restart
   ```

该工作流目前发布的是 x86_64 Linux helper。ARM、RISC-V、非 Linux，或目标系统无法
运行这两个产物时，需要在自己的 fork 中扩展 CI target，或在相应平台本地构建。

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
