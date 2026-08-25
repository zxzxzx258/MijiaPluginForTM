#!/usr/bin/env fish

set -l repo_dir (cd (dirname (status filename)); and pwd)
or exit 1

if not command -q cargo
    echo "未找到 cargo；请先安装 Rust 工具链。" >&2
    exit 1
end

set -l config_root "$HOME/.config"
if set -q XDG_CONFIG_HOME; and test -n "$XDG_CONFIG_HOME"
    set config_root "$XDG_CONFIG_HOME"
end

set -l dms_config_dir "$config_root/DankMaterialShell"
set -l plugin_dir "$dms_config_dir/plugins/MijiaNetworkPower"
set -l config_path "$dms_config_dir/mijia-network-power.json"

cd "$repo_dir"
or exit 1

cargo build --release --bin mijia-power-helper
or exit 1

mkdir -p "$plugin_dir"
or exit 1

install -Dm644 dms-plugin/plugin.json "$plugin_dir/plugin.json"
and install -Dm644 dms-plugin/MijiaNetworkPowerDaemon.qml "$plugin_dir/MijiaNetworkPowerDaemon.qml"
and install -Dm644 dms-plugin/MijiaNetworkPowerWidget.qml "$plugin_dir/MijiaNetworkPowerWidget.qml"
and install -Dm644 dms-plugin/MijiaNetworkPowerSettings.qml "$plugin_dir/MijiaNetworkPowerSettings.qml"
and install -Dm644 dms-plugin/MijiaDeviceEditor.qml "$plugin_dir/MijiaDeviceEditor.qml"
and install -Dm644 dms-plugin/assets/xiaomi.svg "$plugin_dir/assets/xiaomi.svg"
and install -Dm755 target/release/mijia-power-helper "$plugin_dir/mijia-power-helper"
or exit 1

if not test -e "$config_path"
    install -Dm600 config.example.json "$config_path"
else
    chmod 600 "$config_path"
end

echo "DMS 插件已安装到：$plugin_dir"
echo "米家配置文件：$config_path"
echo "到 DMS 设置的插件页启用“米家功率与网速”，并点击面板组件添加或编辑设备。"
