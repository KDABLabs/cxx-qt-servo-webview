// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Layouts 1.12

import com.kdab.servo 1.0

ColumnLayout {
    id: root

    signal goBack()
    signal goForward()
    signal infoPanelRequest()
    signal kdabPanelRequest()
    signal warningPanelRequest()
    signal urlRequest(url requestedUrl)

    property alias canGoBack: backButton.enabled
    property alias canGoForward: forwardButton.enabled
    property url faviconUrl
    property url webViewUrl

    spacing: 0

    // Toolbar of controls for Servo
    RowLayout {
        Layout.preferredHeight: 64
        Layout.fillWidth: true
        spacing: 0

        component ServoButton: ToolButton {
            hoverEnabled: false
            Layout.alignment: Qt.AlignVCenter
            Layout.preferredHeight: 64
            Layout.preferredWidth: 64
            icon.height: 32
            icon.width: 32
            opacity: enabled ? 1.0 : 0.5
        }

        component ServoSeparator: Rectangle {
            color: Qt.rgba(0, 0, 0, 0.3)
            Layout.alignment: Qt.AlignVCenter
            Layout.preferredHeight: 24
            Layout.preferredWidth: 2
        }

        ServoButton {
            id: backButton
            icon.source: "../images/arrow-back.png"
            icon.height: 48
            icon.width: 48

            onClicked: root.goBack()
        }

        ServoButton {
            id: forwardButton
            icon.source: "../images/arrow-forward.png"
            icon.height: 48
            icon.width: 48

            onClicked: root.goForward()
        }

        Rectangle {
            Layout.alignment: Qt.AlignVCenter
            Layout.fillWidth: true
            Layout.preferredHeight: 48
            radius: height / 2
            color: Qt.rgba(0, 0, 0, 0.1)

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 12
                anchors.rightMargin: 24
                spacing: 12

                Rectangle {
                    Layout.preferredHeight: 32
                    Layout.preferredWidth: 32
                    Layout.alignment: Qt.AlignVCenter
                    radius: 16

                    Image {
                        anchors.centerIn: parent
                        height: 24
                        source: root.faviconUrl === "https://localhost/emptyfavicon.ico" ? "" : root.faviconUrl
                        sourceSize.height: 24
                        sourceSize.width: 24
                        width: 24
                    }
                }

                Label {
                    elide: Text.ElideRight
                    font.pixelSize: 24
                    Layout.alignment: Qt.AlignVCenter
                    Layout.fillWidth: true
                    // Fake any local urls to an equivalent real url
                    text: root.webViewUrl.toString().replace("http://0.0.0.0:8001/", "https://")

                    MouseArea {
                        anchors.fill: parent
                        onClicked: root.warningPanelRequest()
                    }
                }
            }
        }

        ServoButton {
            icon.source: "../images/favicon-servo.png"

            onClicked: root.urlRequest("http://0.0.0.0:8001/servo.org/")
        }

        ServoSeparator {}

        ServoButton {
            icon.source: "../images/favicon-rust.png"

            onClicked: root.urlRequest("http://0.0.0.0:8001/www.rust-lang.org/")
        }

        ServoSeparator {}

        ServoButton {
            icon.source: "../images/favicon-kdab.png"

            onClicked: root.infoPanelRequest()
            onPressed: kdabTimer.restart()
            onReleased: kdabTimer.stop()

            Timer {
                id: kdabTimer
                interval: 5000

                onTriggered: root.kdabPanelRequest()
            }
        }

        ServoSeparator {}

        ServoButton {
            icon.source: "../images/favicon-qt.png"

            onClicked: root.infoPanelRequest()
        }
    }

    Rectangle {
        color: Qt.rgba(0, 0, 0, 0.3)
        Layout.fillWidth: true
        Layout.preferredHeight: 2
    }
}
