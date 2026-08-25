# COSMIC Monitor

Rust 面板小程序基座，用于在 COSMIC panel 中显示实时上下行网速，并可通过局域网 miIO 读取米家设备功率。它参考了 TrafficMonitor 的“可扩展指标 + 面板显示 + 详情弹窗”形态，但不加载 Windows DLL，也不复制目标仓库中与 Windows 宿主绑定的代码。

- 当前分支：`cosmic`
- 当前版本：`0.1.0`
- 作者与维护者：`LinusLIU`

## 项目家族与分支

本分支是从 Windows TrafficMonitor 插件概念重写而来的 COSMIC 原生实现。各分支
共享 miIO 功率读取思路，但面向不同宿主，二进制和配置文件不能互换：

| 分支 | 版本/阶段 | 平台 | 关系与功能 |
| --- | --- | --- | --- |
| [`main`](https://github.com/zxzxzx258/MijiaPluginForTM/tree/main) | v1.0.0 基线 | Windows / TrafficMonitor | 单设备功率、历史记录、设置对话框，并提供完整 IP/token 获取教程 |
| [`feature/multi-device-power`](https://github.com/zxzxzx258/MijiaPluginForTM/tree/feature/multi-device-power) | 多设备开发线 | Windows / TrafficMonitor | 将 Windows 插件扩展为多个独立显示项和可迁移配置 |
| [`cosmic`](https://github.com/zxzxzx258/MijiaPluginForTM/tree/cosmic) | v0.1.0 | Linux / COSMIC | 当前分支；网速、自动物理网卡选择、多设备功率和主题单色图标 |
| [`DMS`](https://github.com/zxzxzx258/MijiaPluginForTM/tree/DMS) | v0.2.3 | Linux / DMS | 后续 DMS 集成；原生设置页、可折叠详情和逐设备状态栏选择 |

功能迭代顺序为：`main` 单设备基线 -> Windows 多设备开发线 -> `cosmic` 原生 Linux
面板 -> `DMS` 的完整 DMS 集成。获取 IP/token 可参考
[`main` README](https://github.com/zxzxzx258/MijiaPluginForTM/tree/main)，需要 DMS 时应直接使用
[`DMS` 分支](https://github.com/zxzxzx258/MijiaPluginForTM/tree/DMS)。

## 当前实现

- 由 `X-CosmicApplet=true` 的 XDG desktop 条目注册为 COSMIC applet。
- 读取 `/proc/net/dev`，以采样间隔计算接口接收/发送速率；自动模式优先选择默认路由对应的物理网卡，并排除 TUN/bridge 等虚拟接口。
- 米家读取走 UDP `54321`、hello 握手、AES-128-CBC 和 `get_properties`；默认属性为 `siid=11/piid=2`，但配置可覆盖。
- 面板单行显示下载、上传和每台设备的名称与功率，垂直高度与其他 COSMIC panel 图标对齐。
- 每台米家设备独立采样和显示错误；单台设备超时不会覆盖其他设备的读数。
- 没有米家配置时仍可独立使用网络监控，不会扫描局域网。
- 点击面板后可在弹窗中选择“自动”或固定到当前激活的物理网卡，选择会立即持久化；有线网络成为默认路由后，自动模式会随之切换。

## 构建与用户级安装

需要 Rust 1.85+、Wayland/COSMIC 运行时和可访问 crates.io/Git 仓库的网络：

```fish
cargo test
cargo build --release
install -Dm755 target/release/cosmic-applet-monitor ~/.local/bin/cosmic-applet-monitor
install -Dm644 data/io.github.cosmic.Monitor.desktop ~/.local/share/applications/io.github.cosmic.Monitor.desktop
install -Dm644 data/icons/hicolor/scalable/apps/io.github.cosmic.Monitor-symbolic.svg ~/.local/share/icons/hicolor/scalable/apps/io.github.cosmic.Monitor-symbolic.svg
```

然后在 COSMIC 设置的面板项目中加入 `COSMIC Monitor`。不同发行版的面板刷新方式不同，重新登录是最可靠的发现方式。

插件使用单色 Xiaomi symbolic 图标；COSMIC 会根据 Dark/Light 主题自动调整图标颜色。图标由仓库所有者提供，用于本非商业项目。

## 配置与令牌边界

配置路径为 `$XDG_CONFIG_HOME/cosmic-applet-monitor/config.json`，未设置时为 `~/.config/cosmic-applet-monitor/config.json`。复制 `config.example.json` 后为 `mijia` 数组中的每台设备填入稳定 `id`、局域网 IP、16 字节十六进制 token 和实际属性编号，并确保文件权限为 `0600`：

`interface` 为 `null` 时使用自动模式；也可以写入 `"wlan0"`、`"enp14s0"` 等接口名固定选择。通常直接在 applet 弹窗中修改即可。

```fish
mkdir -p ~/.config/cosmic-applet-monitor
cp config.example.json ~/.config/cosmic-applet-monitor/config.json
chmod 600 ~/.config/cosmic-applet-monitor/config.json
```

程序不会自动扫描局域网或回显凭据；不要把真实 token 提交到 Git、终端日志或问题报告。仓库根目录的 `config.json` 和 `config.local.json` 已被忽略，实际运行配置仍应保存在上述用户配置目录中。

## 局限

米家固件可能禁用本地 miIO，且不同型号的功率属性编号不同；没有真实设备配置时只能验证协议编码和错误处理，不能宣称硬件连接成功。米家设备必须与本机处于可达的同一局域网。

## 版本迭代

- `0.1.0`：完成 COSMIC applet 注册、网速采样、自动/固定物理网卡选择、多设备
  miIO 功率读取和浅色/深色主题单色 Xiaomi 图标。
- 后续更完整的 DMS 设置页、折叠设备管理和逐设备状态栏选择在
  [`DMS` v0.2.3](https://github.com/zxzxzx258/MijiaPluginForTM/tree/DMS) 中继续演进。
