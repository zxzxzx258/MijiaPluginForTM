import QtQuick
import QtCore
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Widgets

Item {
    id: root

    property var requestPowerRefresh: null
    property alias expanded: deviceSection.expanded
    property bool deviceSelectionEnabled: true
    readonly property string configPath: Paths.strip(StandardPaths.writableLocation(StandardPaths.ConfigLocation)) + "/DankMaterialShell/mijia-network-power.json"

    property var configDocument: ({})
    property var devices: []
    property string status: "正在读取设备配置..."
    property bool editorOpen: false
    property int editingIndex: -1
    property string editName: ""
    property string editIp: ""
    property string editToken: ""
    property string editModel: "local.mijia.device"
    property string editSiid: "11"
    property string editPiid: "2"
    property string editorError: ""

    implicitHeight: deviceSection.implicitHeight
    height: implicitHeight

    function clone(value) {
        return JSON.parse(JSON.stringify(value));
    }

    function loadConfig(raw) {
        try {
            const parsed = JSON.parse(raw);
            configDocument = parsed && typeof parsed === "object" ? parsed : {};
            devices = Array.isArray(configDocument.mijia) ? clone(configDocument.mijia) : [];
            status = devices.length > 0 ? "已加载 " + devices.length + " 台设备" : "尚未添加米家设备";
        } catch (error) {
            configDocument = {};
            devices = [];
            status = "配置文件格式无效";
        }
    }

    function beginAdd() {
        deviceSection.expanded = true;
        editingIndex = -1;
        editName = "米家设备 " + (devices.length + 1);
        editIp = "";
        editToken = "";
        editModel = "local.mijia.device";
        editSiid = "11";
        editPiid = "2";
        editorError = "";
        editorOpen = true;
    }

    function beginEdit(index) {
        const device = devices[index];
        if (!device)
            return;
        deviceSection.expanded = true;
        editingIndex = index;
        editName = String(device.name || "米家设备 " + (index + 1));
        editIp = String(device.ip || "");
        editToken = String(device.token || "");
        editModel = String(device.model || "local.mijia.device");
        editSiid = String(device.power_siid || 11);
        editPiid = String(device.power_piid || 2);
        editorError = "";
        editorOpen = true;
    }

    function validIpv4(value) {
        const parts = value.split(".");
        return parts.length === 4 && parts.every(part => /^\d+$/.test(part) && Number(part) >= 0 && Number(part) <= 255);
    }

    function saveEditor() {
        const name = editName.trim();
        const ip = editIp.trim();
        const token = editToken.trim().toLowerCase();
        const model = editModel.trim() || "local.mijia.device";
        const siid = Number(editSiid);
        const piid = Number(editPiid);
        if (!name) {
            editorError = "请填写设备名称";
            return;
        }
        if (!validIpv4(ip)) {
            editorError = "请输入有效的 IPv4 地址";
            return;
        }
        if (!/^[0-9a-f]{32}$/.test(token)) {
            editorError = "token 必须为 32 位十六进制字符";
            return;
        }
        if (!Number.isInteger(siid) || siid < 1 || !Number.isInteger(piid) || piid < 1) {
            editorError = "SIID 和 PIID 必须是正整数";
            return;
        }

        const updated = devices.slice();
        const id = editingIndex >= 0 ? String(updated[editingIndex].id || "mijia-" + (editingIndex + 1)) : "mijia-" + Date.now();
        const device = {
            id: id,
            name: name,
            ip: ip,
            token: token,
            model: model,
            power_siid: siid,
            power_piid: piid,
            show_in_bar: editingIndex >= 0 ? deviceShownInBar(updated[editingIndex]) : true
        };
        if (editingIndex >= 0)
            updated[editingIndex] = device;
        else
            updated.push(device);
        devices = updated;
        editorOpen = false;
        saveConfig();
    }

    function removeDevice(index) {
        const updated = devices.slice();
        updated.splice(index, 1);
        devices = updated;
        saveConfig();
    }

    function deviceShownInBar(device) {
        return !device || device.show_in_bar !== false;
    }

    function setDeviceShownInBar(index, checked) {
        if (!deviceSelectionEnabled || index < 0 || index >= devices.length)
            return;
        const updated = clone(devices);
        updated[index].show_in_bar = checked;
        devices = updated;
        saveConfig();
    }

    function saveConfig() {
        const document = clone(configDocument);
        document.mijia = clone(devices);
        configDocument = document;
        configFile.setText(JSON.stringify(document, null, 2) + "\n");
    }

    FileView {
        id: configFile
        path: root.configPath
        blockLoading: true
        blockWrites: true
        atomicWrites: true
        printErrors: false
        watchChanges: true

        onLoaded: root.loadConfig(text())
        onLoadFailed: error => {
            root.configDocument = {};
            root.devices = [];
            root.status = "配置文件尚未准备好";
        }
        onSaved: {
            root.status = "设备配置已保存";
            permissionsProcess.running = true;
        }
        onSaveFailed: error => {
            root.status = "无法保存设备配置";
        }
    }

    Process {
        id: permissionsProcess
        command: ["chmod", "644", root.configPath]
        onExited: exitCode => {
            if (exitCode === 0 && root.requestPowerRefresh)
                root.requestPowerRefresh();
            else if (exitCode !== 0)
                root.status = "无法收紧配置文件权限";
        }
    }

    MijiaCollapsibleSection {
        id: deviceSection
        width: parent.width
        title: "米家设备（" + root.devices.length + "）"
        description: root.status
        expanded: false
        Item {
            width: parent.width
            height: addButton.height

            DankButton {
                id: addButton
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                text: "添加设备"
                iconName: "add"
                buttonHeight: 36
                horizontalPadding: Theme.spacingM
                onClicked: root.beginAdd()
            }
        }

        Repeater {
            model: root.devices

            delegate: StyledRect {
                required property var modelData
                required property int index
                width: parent.width
                implicitHeight: deviceDetails.implicitHeight + Theme.spacingM * 2
                radius: Theme.cornerRadius
                color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)
                border.width: 0

                Column {
                    id: deviceDetails
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.margins: Theme.spacingM
                    spacing: Theme.spacingXS

                    Row {
                        width: parent.width
                        spacing: Theme.spacingS

                        Column {
                            width: parent.width - barToggle.width - editAction.width - deleteAction.width - parent.spacing * 3
                            spacing: Theme.spacingXXS

                            StyledText {
                                text: modelData.name || "米家设备"
                                width: parent.width
                                color: Theme.surfaceText
                                font.pixelSize: Theme.fontSizeMedium
                                elide: Text.ElideRight
                            }

                            StyledText {
                                text: modelData.ip || ""
                                width: parent.width
                                color: Theme.surfaceVariantText
                                font.pixelSize: Theme.fontSizeSmall
                                elide: Text.ElideRight
                            }
                        }

                        DankToggle {
                            id: barToggle

                            enabled: root.deviceSelectionEnabled
                            checked: root.deviceShownInBar(modelData)
                            anchors.verticalCenter: parent.verticalCenter
                            onToggled: checked => root.setDeviceShownInBar(index, checked)
                        }

                        DankActionButton {
                            id: editAction
                            iconName: "edit"
                            tooltipText: "编辑设备"
                            onClicked: root.beginEdit(index)
                        }

                        DankActionButton {
                            id: deleteAction
                            iconName: "delete"
                            iconColor: Theme.error
                            tooltipText: "删除设备"
                            onClicked: root.removeDevice(index)
                        }
                    }
                }
            }
        }

        StyledRect {
            width: parent.width
            implicitHeight: editorContent.implicitHeight + Theme.spacingM * 2
            radius: Theme.cornerRadius
            color: Theme.withAlpha(Theme.surfaceContainerHighest, Theme.popupTransparency)
            border.width: 0
            visible: root.editorOpen

            Column {
                id: editorContent
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.margins: Theme.spacingM
                spacing: Theme.spacingS

                StyledText {
                    text: root.editingIndex >= 0 ? "编辑米家设备" : "添加米家设备"
                    color: Theme.surfaceText
                    font.pixelSize: Theme.fontSizeMedium
                    font.weight: Font.Medium
                }

                DankTextField {
                    width: parent.width
                    labelText: "设备名称"
                    text: root.editName
                    onTextEdited: root.editName = text
                }

                DankTextField {
                    width: parent.width
                    labelText: "IP 地址"
                    text: root.editIp
                    onTextEdited: root.editIp = text
                }

                DankTextField {
                    width: parent.width
                    labelText: "miIO token"
                    text: root.editToken
                    maximumLength: 32
                    showPasswordToggle: true
                    echoMode: passwordVisible ? TextInput.Normal : TextInput.Password
                    onTextEdited: root.editToken = text
                }

                DankTextField {
                    width: parent.width
                    labelText: "设备型号"
                    text: root.editModel
                    onTextEdited: root.editModel = text
                }

                Row {
                    width: parent.width
                    spacing: Theme.spacingS

                    DankTextField {
                        width: (parent.width - parent.spacing) / 2
                        labelText: "功率 SIID"
                        text: root.editSiid
                        onTextEdited: root.editSiid = text
                    }

                    DankTextField {
                        width: (parent.width - parent.spacing) / 2
                        labelText: "功率 PIID"
                        text: root.editPiid
                        onTextEdited: root.editPiid = text
                    }
                }

                StyledText {
                    width: parent.width
                    text: root.editorError
                    visible: text !== ""
                    color: Theme.error
                    font.pixelSize: Theme.fontSizeSmall
                    wrapMode: Text.WordWrap
                }

                Row {
                    spacing: Theme.spacingS

                    DankButton {
                        text: "保存"
                        onClicked: root.saveEditor()
                    }

                    DankButton {
                        text: "取消"
                        onClicked: root.editorOpen = false
                    }
                }
            }
        }
    }
}
