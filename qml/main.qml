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
    color: "white"
    title: webView.title
    visibility: Window.FullScreen
    visible: true
    height: 800
    width: 1280

    // Ensure we have the right size for the EW demo to avoid glitches
    Item {
        height: 800
        width: 1280

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            ServoToolbar {
                id: toolbar
                canGoBack: webView.canGoBack
                canGoForward: webView.canGoForward
                faviconUrl: webView.faviconUrl
                webViewUrl: webView.url
                Layout.fillWidth: true

                onGoBack: webView.goBack()
                onGoForward: webView.goForward()
                onInfoPanelRequest: infoPanel.show()
                onKdabPanelRequest: kdabPanel.show()
                onWarningPanelRequest: warningPanel.show()
                onUrlRequest: (requestedUrl) => webView.url = requestedUrl
            }

            // Servo webview
            ServoWebView {
                id: webView
                Layout.fillHeight: true
                Layout.fillWidth: true
                navigationAllowed: false
                url: "http://0.0.0.0:8001/servo.org/"

                onBlockedNavigationRequest: (blockedUrl) => {
                    // Allow for navigation to any offline sites
                    if (blockedUrl.toString().startsWith("http://0.0.0.0:8001/")) {
                        webView.url = blockedUrl;
                    } else {
                        warningPanel.show();
                    }
                }
            }
        }

        // Progress bar at the bottom overlaying the Servo WebView
        // so that we don't have a flicker when it's hidden as this doesn't cause a resize
        ProgressBar {
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: 10
            indeterminate: true
            visible: webView.loading
        }

        InfoPanel {
            id: infoPanel
            anchors.fill: parent
        }

        WarningPanel {
            id: warningPanel
            anchors.fill: parent
        }

        KdabPanel {
            id: kdabPanel
            anchors.fill: parent
        }
    }
}
