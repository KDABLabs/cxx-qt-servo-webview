// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Layouts 1.12
import QtQuick.Window 2.12

import com.kdab.servo 1.0

Window {
    id: root
    height: 10 + 16 + 10 + 20 + 10 + 400 + 10
    title: qsTr("Servo CXX-Qt")
    visible: true
    width: 420

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        RowLayout {
            height: 16
            Layout.fillWidth: true
            spacing: 10

            Image {
                height: 16
                source: webView.faviconUrl
                sourceSize.height: 16
                sourceSize.width: 16
                width: 16
                visible: status === Image.Ready
            }

            Label {
                elide: Text.ElideRight
                font.pixelSize: 16
                Layout.fillWidth: true
                text: webView.title
            }

            Item {
                Layout.fillWidth: true
            }
        }

        RowLayout {
            height: 20
            Layout.fillWidth: true
            spacing: 10

            TextField {
                id: textInputUrl
                Layout.fillWidth: true
                text: webView.url
                placeholderText: qsTr("Url...")
                onAccepted: {
		   webView.url = textInputUrl.text
                }
            }

            Button {
                text: qsTr("Go")

                onClicked: webView.url = textInputUrl.text
            }
        }

        ServoWebView {
            id: webView
            Layout.fillHeight: true
            Layout.fillWidth: true
            url: "https://servo.org/"
        }
    }

    BusyIndicator {
        anchors.centerIn: parent
        running: webView.loading
        visible: running
    }
}
