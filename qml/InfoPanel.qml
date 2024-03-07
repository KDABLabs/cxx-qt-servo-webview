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

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 50

        Text {
            color: "white"
            font.family: "Open Sans"
            font.italic: true
            font.pixelSize: 45
            text: qsTr("CXX-Qt - Servo Demo")
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 50

            Image {
                fillMode: Image.PreserveAspectFit
                Layout.maximumHeight: 200
                Layout.maximumWidth: 200
                source: "../images/kdab-logo.png"
            }

            Text {
                color: "white"
                font.family: "Open Sans"
                horizontalAlignment: Text.AlignJustify
                font.pixelSize: 18
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                text: qsTr("<p><b>CXX-Qt - Safe Rust bindings for Qt</b></p>
                <p>KDAB is working to bridge between Rust and Qt. CXX-Qt allows you to define Qt objects from Rust, so that the business logic can be written in Rust while Qt handles the frontend.</p>
                <p>In this Servo demo, CXX-Qt provides a web rendering engine (Servo) as a component to QML.</p>")
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 50

            Image {
                fillMode: Image.PreserveAspectFit
                Layout.maximumHeight: 200
                Layout.maximumWidth: 200
                source: "../images/servo-color-negative-no-container.svg"
            }

            Text {
                color: "white"
                font.family: "Open Sans"
                horizontalAlignment: Text.AlignJustify
                font.pixelSize: 18
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                text:"<p><b>Servo</b></p>
                <p>A web rendering engine written in Rust, with WebGL and WebGPU support, and adaptable to desktop, mobile, and embedded applications.<p>"
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 50

            Image {
                fillMode: Image.PreserveAspectFit
                Layout.maximumHeight: 200
                Layout.maximumWidth: 200
                source: "../images/rust-logo-white.png"
            }

            Text {
                color: "white"
                font.family: "Open Sans"
                horizontalAlignment: Text.AlignJustify
                font.pixelSize: 18
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                text:"<p><b>Rust</b></p>
                <p>A programming language empowering everyone to build reliable and efficient software.</p>
                <p>Blazingly fast and memory-efficient: with no runtime or garbage collector, it can power performance-critical services, run on embedded devices, and easily integrate with other languages.</p>
                <p>Rust's rich type system and ownership model guarantee memory-safety and thread-safety - enabling you to eliminate many classes of bugs at compile-time.</p>"
            }
        }
    }

    MouseArea {
        anchors.fill: parent

        onClicked: root.hide()
    }
}
