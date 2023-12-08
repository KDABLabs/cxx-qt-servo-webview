import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Window 2.12

import com.kdab.servo 1.0

Window {
    height: 800
    title: qsTr("Hello World")
    visible: true
    width: 800

    ServoWebView {
        //anchors.centerIn: parent
        //height: 600
        //width: 600
        url: "https://kdab.com"
    }
}
