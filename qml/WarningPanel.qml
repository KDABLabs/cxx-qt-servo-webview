// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Layouts 1.12

Rectangle {
    id: root

    function hide() {
        opacity = 0.0;
    }

    function show() {
        opacity = 1.0;
    }

    color: Qt.rgba(0, 0, 0, 0.85)
    opacity: 0.0
    visible: opacity !== 0.0

    Behavior on opacity {
        NumberAnimation {

        }
    }

    Text {
        anchors.centerIn: parent
        color: "white"
        font.family: "Open Sans"
        font.pixelSize: 18
        text: qsTr("This demo is running offline, so navigation to other sites is not possible")
    }

    MouseArea {
        anchors.fill: parent

        onClicked: root.hide()
    }
}
