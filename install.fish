#!/usr/bin/env fish

argparse 'helper=' -- $argv
or exit 2

if test (count $argv) -ne 0
    echo "用法：fish install.fish [--helper /path/to/mijia-power-helper]" >&2
    exit 2
end

set -l repo_dir (cd (dirname (status filename)); and pwd)
or exit 1

set -l helper_source ""
if set -q _flag_helper
    set helper_source (realpath "$_flag_helper")
    or begin
        echo "找不到预编译 helper：$_flag_helper" >&2
        exit 1
    end
    if not test -f "$helper_source"
        echo "预编译 helper 不是普通文件：$helper_source" >&2
        exit 1
    end
else
    if not command -q cargo
        echo "未找到 cargo。可安装 Rust，或使用 --helper 指定 GitHub Actions 产物。" >&2
        exit 1
    end
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

if test -z "$helper_source"
    cargo build --release --bin mijia-power-helper
    or exit 1
    set helper_source "$repo_dir/target/release/mijia-power-helper"
else
    echo "使用预编译 helper：$helper_source"
end

mkdir -p "$plugin_dir"
or exit 1

install -Dm644 dms-plugin/plugin.json "$plugin_dir/plugin.json"
and install -Dm644 dms-plugin/MijiaNetworkPowerDaemon.qml "$plugin_dir/MijiaNetworkPowerDaemon.qml"
and install -Dm644 dms-plugin/MijiaNetworkPowerWidget.qml "$plugin_dir/MijiaNetworkPowerWidget.qml"
and install -Dm644 dms-plugin/MijiaNetworkPowerSettings.qml "$plugin_dir/MijiaNetworkPowerSettings.qml"
and install -Dm644 dms-plugin/MijiaDeviceEditor.qml "$plugin_dir/MijiaDeviceEditor.qml"
and install -Dm644 dms-plugin/MijiaCollapsibleSection.qml "$plugin_dir/MijiaCollapsibleSection.qml"
and install -Dm644 dms-plugin/assets/xiaomi.svg "$plugin_dir/assets/xiaomi.svg"
and install -Dm755 "$helper_source" "$plugin_dir/mijia-power-helper"
or exit 1

if not test -e "$config_path"
    install -Dm600 config.example.json "$config_path"
else
    chmod 600 "$config_path"
end

echo "DMS 插件已安装到：$plugin_dir"
echo "米家配置文件：$config_path"
echo "到 DMS 设置的插件页启用“米家功率与网速”，并点击面板组件添加或编辑设备。"
