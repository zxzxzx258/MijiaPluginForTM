import QtQuick
import qs.Common
import qs.Widgets

Column {
    id: root

    required property string title
    property string description: ""
    property bool expanded: false
    default property alias content: body.data

    spacing: 0

    StyledRect {
        id: header

        width: root.width
        height: 44
        radius: Theme.cornerRadius
        color: headerMouse.containsMouse ? Theme.surfaceContainerHigh : "transparent"
        border.width: 0

        Row {
            anchors.fill: parent
            anchors.leftMargin: Theme.spacingM
            anchors.rightMargin: Theme.spacingM
            spacing: Theme.spacingS

            StyledText {
                width: Math.max(0, parent.width - expandIcon.width - parent.spacing)
                text: root.title
                color: Theme.surfaceText
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.Medium
                elide: Text.ElideRight
                wrapMode: Text.NoWrap
                anchors.verticalCenter: parent.verticalCenter
            }

            DankIcon {
                id: expandIcon

                name: "expand_more"
                size: Theme.iconSizeSmall
                color: Theme.surfaceVariantText
                rotation: root.expanded ? 180 : 0
                anchors.verticalCenter: parent.verticalCenter

                Behavior on rotation {
                    NumberAnimation {
                        duration: Theme.shortDuration
                        easing.type: Theme.standardEasing
                    }
                }
            }
        }

        MouseArea {
            id: headerMouse

            anchors.fill: parent
            hoverEnabled: true
            preventStealing: true
            propagateComposedEvents: false
            cursorShape: Qt.PointingHandCursor
            onPressed: mouse => mouse.accepted = true
            onClicked: root.expanded = !root.expanded
        }
    }

    Item {
        id: bodyWrapper

        width: root.width
        height: root.expanded ? body.implicitHeight + Theme.spacingS : 0
        clip: true

        Behavior on height {
            NumberAnimation {
                duration: Theme.shortDuration
                easing.type: Theme.standardEasing
            }
        }

        Column {
            id: body

            y: Theme.spacingS
            width: parent.width
            spacing: Theme.spacingS
            opacity: root.expanded ? 1 : 0

            StyledText {
                width: parent.width
                visible: root.description !== ""
                text: root.description
                color: Theme.surfaceVariantText
                font.pixelSize: Theme.fontSizeSmall
                wrapMode: Text.WordWrap
            }
        }
    }
}
