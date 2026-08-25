import QtQuick
import QtCore
import Quickshell.Io
import qs.Modules.Plugins
import qs.Widgets

PluginComponent {
    id: root

    readonly property string configPath: StandardPaths.writableLocation(StandardPaths.ConfigLocation) + "/DankMaterialShell/mijia-network-power.json"
    readonly property string helperPath: {
        const pluginPath = pluginService?.getPluginPath(pluginId) || "";
        return pluginPath ? pluginPath + "/mijia-power-helper" : "mijia-power-helper";
    }
    readonly property int refreshSeconds: Math.max(5, Number(pluginData.powerRefreshSeconds || 15));

    property string latestStdout: ""

    PluginGlobalVar {
        id: powerReadings
        varName: "readings"
        defaultValue: root.emptyReadings()
    }

    function emptyReadings() {
        return {
            devices: [],
            configuredDevices: 0,
            error: "正在读取米家设备...",
            updatedAt: 0
        };
    }

    function publish(value) {
        powerReadings.set(value);
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

    onConfigPathChanged: probeNow()
    onRefreshSecondsChanged: pollTimer.restart()
    Component.onCompleted: {
        publish(emptyReadings());
        Qt.callLater(probeNow);
    }
}
