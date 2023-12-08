import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Layouts 1.12
import QtQuick.Window 2.12

import com.kdab.servo 1.0

Window {
    id: root
    height: 800
    title: qsTr("Servo CXX-Qt")
    visible: true
    width: 800

    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        Label {
            elide: Text.ElideRight
            Layout.fillWidth: true
            text: webView.title
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            TextField {
                id: textInputUrl
                Layout.fillWidth: true
                text: webView.url
            }

            Button {
                text: qsTr("Go")

                onClicked: webView.url = textInputUrl.text
            }
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }

    ServoWebView {
        id: webView
        //anchors.centerIn: parent
        //height: 600
        //width: 600
        url: "file:///var/home/andrew/Projects/servo/servo/tests/html/about-mozilla.html"
    }
}
