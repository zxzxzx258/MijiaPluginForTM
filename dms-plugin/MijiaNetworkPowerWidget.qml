import QtQuick
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Modules.Plugins
import qs.Services
import qs.Widgets

PluginComponent {
    id: root

    readonly property bool showNetwork: pluginData.showNetwork !== false
    readonly property bool showTotalPower: pluginData.showTotalPower === true
    readonly property bool showDevicePowerInBar: pluginData.showDevicePowerInBar !== false
    readonly property var powerData: powerReadings.value || emptyReadings()
    readonly property var devices: Array.isArray(powerData.devices) ? powerData.devices : []
    readonly property var barDevices: showDevicePowerInBar ? devices.filter(device => device.show_in_bar !== false) : []
    readonly property var successfulDevices: devices.filter(device => device.watts !== null && device.watts !== undefined)
    readonly property real totalWatts: successfulDevices.reduce((sum, device) => sum + Number(device.watts), 0)
    readonly property string logoPath: {
        const pluginPath = pluginService?.getPluginPath(pluginId) || "";
        return pluginPath ? pluginPath + "/assets/xiaomi.svg" : "";
    }

    PluginGlobalVar {
        id: powerReadings
        varName: "readings"
        defaultValue: root.emptyReadings()
    }

    PluginGlobalVar {
        id: refreshRequest
        varName: "refreshRequest"
        defaultValue: 0
    }

    function emptyReadings() {
        return {
            devices: [],
            configuredDevices: 0,
            error: "",
            updatedAt: 0
        };
    }

    function scaledNetworkSpeed(bytesPerSecond) {
        const units = ["B/s", "KB/s", "MB/s", "GB/s"];
        const compactUnits = ["B", "K", "M", "G"];
        let value = Math.max(0, Number(bytesPerSecond || 0));
        if (!isFinite(value))
            value = 0;
        let unitIndex = 0;
        while (value >= 1024 && unitIndex < units.length - 1) {
            value /= 1024;
            unitIndex++;
        }
        let rounded = Math.round(value);
        if (rounded >= 1024 && unitIndex < units.length - 1) {
            rounded = Math.round(value / 1024);
            unitIndex++;
        }
        return {
            value: rounded,
            unit: units[unitIndex],
            compactUnit: compactUnits[unitIndex]
        };
    }

    function formatNetworkSpeed(bytesPerSecond) {
        const speed = scaledNetworkSpeed(bytesPerSecond);
        return speed.value + " " + speed.unit;
    }

    function formatCompactNetworkSpeed(bytesPerSecond) {
        const speed = scaledNetworkSpeed(bytesPerSecond);
        return speed.value + speed.compactUnit;
    }

    function formatWatts(watts) {
        const value = Number(watts || 0);
        return value < 10 ? value.toFixed(1) + " W" : value.toFixed(0) + " W";
    }

    function statusText() {
        if (powerData.error)
            return powerData.error;
        if (powerData.configuredDevices === 0)
            return "未配置米家设备";
        if (successfulDevices.length === 0)
            return "设备暂不可达";
        if (successfulDevices.length < powerData.configuredDevices)
            return "部分设备暂不可达";
        return "所有设备已更新";
    }

    function requestRefresh() {
        refreshRequest.set(Date.now());
    }

    Component.onCompleted: DgopService.addRef(["network"])
    Component.onDestruction: DgopService.removeRef(["network"])

    horizontalBarPill: Component {
        Row {
            id: horizontalContent
            spacing: Theme.spacingS

            DankSVGIcon {
                width: root.iconSize
                height: root.iconSize
                source: root.logoPath
                size: root.iconSize
                colorOverride: Theme.widgetIconColor
                anchors.verticalCenter: parent.verticalCenter
                visible: root.showNetwork && root.logoPath !== ""
            }

            DankIcon {
                name: "network_check"
                size: root.iconSize
                color: Theme.widgetIconColor
                anchors.verticalCenter: parent.verticalCenter
                visible: root.showNetwork && root.logoPath === ""
            }

            Row {
                spacing: Theme.spacingXS
                visible: root.showNetwork
                anchors.verticalCenter: parent.verticalCenter

                DankIcon {
                    name: "download"
                    size: root.iconSize - 4
                    color: Theme.info
                    anchors.verticalCenter: parent.verticalCenter
                }

                StyledText {
                    text: root.formatNetworkSpeed(DgopService.networkRxRate)
                    font.pixelSize: Theme.barTextSize(root.barThickness, root.barConfig?.fontScale, root.barConfig?.maximizeWidgetText)
                    color: Theme.widgetTextColor
                    anchors.verticalCenter: parent.verticalCenter
                    elide: Text.ElideNone
                    wrapMode: Text.NoWrap
                }
            }

            Row {
                spacing: Theme.spacingXS
                visible: root.showNetwork
                anchors.verticalCenter: parent.verticalCenter

                DankIcon {
                    name: "upload"
                    size: root.iconSize - 4
                    color: Theme.error
                    anchors.verticalCenter: parent.verticalCenter
                }

                StyledText {
                    text: root.formatNetworkSpeed(DgopService.networkTxRate)
                    font.pixelSize: Theme.barTextSize(root.barThickness, root.barConfig?.fontScale, root.barConfig?.maximizeWidgetText)
                    color: Theme.widgetTextColor
                    anchors.verticalCenter: parent.verticalCenter
                    elide: Text.ElideNone
                    wrapMode: Text.NoWrap
                }
            }

            Row {
                spacing: Theme.spacingXS
                visible: root.showTotalPower
                anchors.verticalCenter: parent.verticalCenter

                DankIcon {
                    name: "electric_bolt"
                    size: root.iconSize - 3
                    color: powerData.error || successfulDevices.length === 0 ? Theme.error : Theme.primary
                    anchors.verticalCenter: parent.verticalCenter
                }

                StyledText {
                    text: successfulDevices.length > 0 ? root.formatWatts(root.totalWatts) : "--"
                    font.pixelSize: Theme.barTextSize(root.barThickness, root.barConfig?.fontScale, root.barConfig?.maximizeWidgetText)
                    color: Theme.widgetTextColor
                    anchors.verticalCenter: parent.verticalCenter
                    elide: Text.ElideNone
                    wrapMode: Text.NoWrap
                }
            }

            Repeater {
                model: root.barDevices

                delegate: Row {
                    required property var modelData
                    spacing: Theme.spacingXS
                    anchors.verticalCenter: parent.verticalCenter

                    DankIcon {
                        name: "power"
                        size: root.iconSize - 5
                        color: modelData.watts === null || modelData.watts === undefined ? Theme.error : Theme.primary
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    StyledText {
                        text: modelData.name + " " + (modelData.watts === null || modelData.watts === undefined ? "--" : root.formatWatts(modelData.watts))
                        font.pixelSize: Theme.barTextSize(root.barThickness, root.barConfig?.fontScale, root.barConfig?.maximizeWidgetText)
                        color: Theme.widgetTextColor
                        anchors.verticalCenter: parent.verticalCenter
                        elide: Text.ElideRight
                        wrapMode: Text.NoWrap
                    }
                }
            }
        }
    }

    verticalBarPill: Component {
        Column {
            id: verticalContent
            spacing: Theme.spacingXXS

            DankSVGIcon {
                width: root.iconSize
                height: root.iconSize
                source: root.logoPath
                size: root.iconSize
                colorOverride: Theme.widgetIconColor
                anchors.horizontalCenter: parent.horizontalCenter
                visible: root.showNetwork && root.logoPath !== ""
            }

            DankIcon {
                name: "network_check"
                size: root.iconSize
                color: Theme.widgetIconColor
                anchors.horizontalCenter: parent.horizontalCenter
                visible: root.showNetwork && root.logoPath === ""
            }

            StyledText {
                text: root.showNetwork ? "D " + root.formatCompactNetworkSpeed(DgopService.networkRxRate) : ""
                font.pixelSize: Theme.barTextSize(root.barThickness, root.barConfig?.fontScale, root.barConfig?.maximizeWidgetText)
                color: Theme.info
                anchors.horizontalCenter: parent.horizontalCenter
                visible: root.showNetwork
            }

            StyledText {
                text: root.showNetwork ? "U " + root.formatCompactNetworkSpeed(DgopService.networkTxRate) : ""
                font.pixelSize: Theme.barTextSize(root.barThickness, root.barConfig?.fontScale, root.barConfig?.maximizeWidgetText)
                color: Theme.error
                anchors.horizontalCenter: parent.horizontalCenter
                visible: root.showNetwork
            }

            DankIcon {
                name: "electric_bolt"
                size: root.iconSize - 3
                color: powerData.error || successfulDevices.length === 0 ? Theme.error : Theme.primary
                anchors.horizontalCenter: parent.horizontalCenter
                visible: root.showTotalPower
            }

            StyledText {
                text: root.showTotalPower && successfulDevices.length > 0 ? root.formatWatts(root.totalWatts) : ""
                font.pixelSize: Theme.barTextSize(root.barThickness, root.barConfig?.fontScale, root.barConfig?.maximizeWidgetText)
                color: Theme.widgetTextColor
                anchors.horizontalCenter: parent.horizontalCenter
                visible: root.showTotalPower
            }
        }
    }

    popoutContent: Component {
        PopoutComponent {
            id: popout
            headerText: "米家功率与网速"
            detailsText: root.statusText()
            showCloseButton: true

            Item {
                width: parent.width
                height: Math.max(100, root.popoutHeight - popout.headerHeight - popout.detailsHeight - Theme.spacingXL)

                Flickable {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingS
                    anchors.rightMargin: Theme.spacingS
                    contentWidth: width
                    contentHeight: content.implicitHeight + Theme.spacingS
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds

                    Column {
                        id: content
                        width: parent.width
                        spacing: Theme.spacingM

                        Row {
                            spacing: Theme.spacingL
                            visible: root.showNetwork

                            Row {
                                spacing: Theme.spacingXS

                                DankIcon {
                                    name: "download"
                                    size: Theme.iconSize
                                    color: Theme.info
                                    anchors.verticalCenter: parent.verticalCenter
                                }

                                StyledText {
                                    text: root.formatNetworkSpeed(DgopService.networkRxRate)
                                    color: Theme.surfaceText
                                    font.pixelSize: Theme.fontSizeMedium
                                }
                            }

                            Row {
                                spacing: Theme.spacingXS

                                DankIcon {
                                    name: "upload"
                                    size: Theme.iconSize
                                    color: Theme.error
                                    anchors.verticalCenter: parent.verticalCenter
                                }

                                StyledText {
                                    text: root.formatNetworkSpeed(DgopService.networkTxRate)
                                    color: Theme.surfaceText
                                    font.pixelSize: Theme.fontSizeMedium
                                }
                            }
                        }

                        StyledRect {
                            width: parent.width
                            implicitHeight: totalRow.implicitHeight + Theme.spacingM * 2
                            radius: Theme.cornerRadius
                            color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)
                            border.width: 0

                            Row {
                                id: totalRow
                                anchors.centerIn: parent
                                spacing: Theme.spacingS

                                DankIcon {
                                    name: "electric_bolt"
                                    size: Theme.iconSize
                                    color: root.powerData.error || root.successfulDevices.length === 0 ? Theme.error : Theme.primary
                                    anchors.verticalCenter: parent.verticalCenter
                                }

                                StyledText {
                                    text: root.successfulDevices.length > 0 ? "总功率 " + root.formatWatts(root.totalWatts) : "暂无功率读数"
                                    color: Theme.surfaceText
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.weight: Font.Medium
                                }
                            }
                        }

                        MijiaCollapsibleSection {
                            id: readingsSection
                            width: parent.width
                            title: "设备功率（" + root.devices.length + "）"
                            description: root.devices.length === 0 ? (root.powerData.error || "尚未配置米家设备") : ""
                            expanded: false

                            Repeater {
                                model: root.devices

                                delegate: StyledRect {
                                    required property var modelData
                                    width: parent.width
                                    implicitHeight: deviceRow.implicitHeight + Theme.spacingM * 2
                                    radius: Theme.cornerRadius
                                    color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)
                                    border.width: 0

                                    Row {
                                        id: deviceRow
                                        anchors.left: parent.left
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.leftMargin: Theme.spacingM
                                        anchors.rightMargin: Theme.spacingM
                                        spacing: Theme.spacingS

                                        DankIcon {
                                            name: "power"
                                            size: Theme.iconSize
                                            color: modelData.watts === null || modelData.watts === undefined ? Theme.error : Theme.primary
                                            anchors.verticalCenter: parent.verticalCenter
                                        }

                                        Column {
                                            width: parent.width - valueText.width - Theme.iconSize - Theme.spacingS * 2
                                            spacing: Theme.spacingXXS

                                            StyledText {
                                                text: modelData.name
                                                color: Theme.surfaceText
                                                font.pixelSize: Theme.fontSizeMedium
                                                elide: Text.ElideRight
                                                width: parent.width
                                            }

                                            StyledText {
                                                text: modelData.error || "设备已连接"
                                                color: modelData.error ? Theme.error : Theme.surfaceVariantText
                                                font.pixelSize: Theme.fontSizeSmall
                                                elide: Text.ElideRight
                                                width: parent.width
                                            }
                                        }

                                        StyledText {
                                            id: valueText
                                            text: modelData.watts === null || modelData.watts === undefined ? "--" : root.formatWatts(modelData.watts)
                                            color: Theme.surfaceText
                                            font.pixelSize: Theme.fontSizeMedium
                                            font.weight: Font.Medium
                                            anchors.verticalCenter: parent.verticalCenter
                                        }
                                    }
                                }
                            }
                        }

                        MijiaDeviceEditor {
                            width: parent.width
                            deviceSelectionEnabled: root.showDevicePowerInBar
                            requestPowerRefresh: root.requestRefresh
                        }

                        DankButton {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: "刷新功率"
                            onClicked: root.requestRefresh()
                        }
                    }
                }
            }
        }
    }

    popoutWidth: 420
    popoutHeight: 640
}
