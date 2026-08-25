import QtQuick
import qs.Common
import qs.Modules.Plugins
import qs.Widgets

PluginSettings {
    id: root
    pluginId: "mijiaNetworkPower"

    StyledText {
        width: parent.width
        text: "面板显示"
        font.pixelSize: Theme.fontSizeLarge
        font.weight: Font.Bold
        color: Theme.surfaceText
    }

    ToggleSetting {
        settingKey: "showNetwork"
        label: "显示网速"
        description: "使用 DMS 原生 DgopService，与内置网速监视器的采样口径一致。"
        defaultValue: true
    }

    ToggleSetting {
        settingKey: "showTotalPower"
        label: "显示总功率"
        description: "开启后在每台设备功率之外额外显示汇总值。"
        defaultValue: false
    }

    ToggleSetting {
        id: deviceSelectionSetting

        settingKey: "showDevicePowerInBar"
        label: "选择面板显示设备"
        description: "开启后可在下方设备卡片中逐台勾选；关闭时不在面板显示单台功率。"
        defaultValue: true
    }

    StyledText {
        width: parent.width
        text: "米家读取"
        font.pixelSize: Theme.fontSizeLarge
        font.weight: Font.Bold
        color: Theme.surfaceText
    }

    SliderSetting {
        settingKey: "powerRefreshSeconds"
        label: "功率刷新间隔"
        description: "每次刷新会并行访问已配置的米家设备。"
        defaultValue: 15
        minimum: 5
        maximum: 300
        unit: " 秒"
        leftIcon: "schedule"
    }

    StyledText {
        width: parent.width
        text: "设备管理"
        font.pixelSize: Theme.fontSizeLarge
        font.weight: Font.Bold
        color: Theme.surfaceText
    }

    MijiaDeviceEditor {
        width: parent.width
        expanded: true
        deviceSelectionEnabled: deviceSelectionSetting.value
        requestPowerRefresh: function() {
            if (root.pluginService)
                root.pluginService.setGlobalVar(root.pluginId, "refreshRequest", Date.now());
        }
    }

    StyledText {
        width: parent.width
        text: "设备 IP、token 和属性编号可在此处或组件详情中添加和修改；凭据仅保存在 ~/.config/DankMaterialShell/mijia-network-power.json。"
        color: Theme.surfaceVariantText
        font.pixelSize: Theme.fontSizeSmall
        wrapMode: Text.WordWrap
    }
}
