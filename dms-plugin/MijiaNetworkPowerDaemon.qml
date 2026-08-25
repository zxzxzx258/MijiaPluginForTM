import QtQuick
import QtCore
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Modules.Plugins

PluginComponent {
    id: root

    readonly property string configPath: Paths.strip(StandardPaths.writableLocation(StandardPaths.ConfigLocation)) + "/DankMaterialShell/mijia-network-power.json"
    readonly property string helperPath: {
        const pluginPath = pluginService?.getPluginPath(pluginId) || "";
        return pluginPath ? pluginPath + "/mijia-power-helper" : "mijia-power-helper";
    }
    readonly property int refreshSeconds: Math.max(5, Number(pluginData.powerRefreshSeconds || 15));

    property string latestStdout: ""

    function emptyReadings() {
        return {
            devices: [],
            configuredDevices: 0,
            error: "正在读取米家设备...",
            updatedAt: 0
        };
    }

    function publish(value) {
        if (pluginService && pluginId)
            pluginService.setGlobalVar(pluginId, "readings", value);
    }

    function probeNow() {
        if (probeProcess.running)
            return;

        latestStdout = "";
        probeProcess.running = true;
    }

    function consumeResult(text) {
        latestStdout = text.trim();
        if (!latestStdout)
            return;

        try {
            const result = JSON.parse(latestStdout);
            const devices = Array.isArray(result.devices) ? result.devices : [];
            publish({
                devices: devices,
                configuredDevices: Number(result.configured_devices || 0),
                error: result.error || "",
                updatedAt: Date.now()
            });
        } catch (error) {
            publish({
                devices: [],
                configuredDevices: 0,
                error: "米家 helper 返回了无效数据",
                updatedAt: Date.now()
            });
        }
    }

    Process {
        id: probeProcess
        command: [root.helperPath, "--config", root.configPath]

        stdout: StdioCollector {
            onStreamFinished: root.consumeResult(text)
        }

        onExited: exitCode => {
            if (exitCode !== 0 && !root.latestStdout) {
                root.publish({
                    devices: [],
                    configuredDevices: 0,
                    error: "无法运行米家 helper，请重新执行安装脚本",
                    updatedAt: Date.now()
                });
            }
            pollTimer.restart();
        }
    }

    Timer {
        id: pollTimer
        interval: root.refreshSeconds * 1000
        repeat: false
        running: true
        onTriggered: root.probeNow()
    }

    Connections {
        target: root.pluginService

        function onGlobalVarChanged(changedPluginId, variableName) {
            if (changedPluginId === root.pluginId && variableName === "refreshRequest")
                root.probeNow();
        }
    }

    IpcHandler {
        target: "mijiaNetworkPower"

        function status(): string {
            const data = root.pluginService?.getGlobalVar(root.pluginId, "readings", root.emptyReadings()) || root.emptyReadings();
            const devices = Array.isArray(data.devices) ? data.devices : [];
            return JSON.stringify({
                configuredDevices: Number(data.configuredDevices || 0),
                readableDevices: devices.filter(device => device.watts !== null && device.watts !== undefined).length,
                error: data.error || "",
                updatedAt: Number(data.updatedAt || 0)
            });
        }

        function refresh(): string {
            root.probeNow();
            return "refresh requested";
        }
    }

    onConfigPathChanged: probeNow()
    onRefreshSecondsChanged: pollTimer.restart()
    Component.onCompleted: {
        publish(emptyReadings());
        Qt.callLater(probeNow);
    }
}
