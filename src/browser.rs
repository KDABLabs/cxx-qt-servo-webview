// SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Andrew Hayzen <andrew.hayzen@kdab.com>
//
// SPDX-License-Identifier: MPL-2.0

use servo::{
    compositing::windowing::EmbedderEvent, embedder_traits::EmbedderMsg,
    TopLevelBrowsingContextId as WebViewId,
};
use std::collections::HashMap;
use std::vec::Drain;

#[derive(Default)]
pub struct WebView {}

#[derive(Default)]
pub(crate) struct QServoBrowserResponse {
    pub(crate) favicon_url: Option<url::Url>,
    pub(crate) present: Option<bool>,
    pub(crate) title: Option<String>,
    pub(crate) loading: Option<bool>,
    pub(crate) url: Option<url::Url>,
    pub(crate) blocked_navigation_request: Option<url::Url>,
    pub(crate) can_go_back: Option<bool>,
    pub(crate) can_go_forward: Option<bool>,
}

#[derive(Default)]
pub(crate) struct QServoBrowser {
    web_views: HashMap<WebViewId, WebView>,
    event_queue: Vec<EmbedderEvent>,
    focused_webview_id: Option<WebViewId>,
}

impl QServoBrowser {
    pub fn webview_id(&self) -> Option<WebViewId> {
        self.focused_webview_id
    }

    pub fn get_events(&mut self) -> Vec<EmbedderEvent> {
        std::mem::take(&mut self.event_queue)
    }

    /// Returns true if the caller needs to manually present a new frame.
    ///
    /// TODO: does this move into the WebView?
    pub fn handle_servo_events(
        &mut self,
        events: Drain<'_, (Option<WebViewId>, EmbedderMsg)>,
        navigation_allowed: bool,
    ) -> QServoBrowserResponse {
        let mut response = QServoBrowserResponse::default();

        for (webview_id, msg) in events {
            match msg {
                // Do not allow for opening a new tab / window
                // Not handling this crashes the webview
                EmbedderMsg::AllowOpeningWebView(ipc) => {
                    ipc.send(false).unwrap();
                }
                EmbedderMsg::AllowNavigationRequest(pipeline_id, url) => {
                    if let Some(_webview_id) = webview_id {
                        self.event_queue
                            .push(EmbedderEvent::AllowNavigationResponse(
                                pipeline_id,
                                navigation_allowed,
                            ));

                        let url = url.into_url();
                        if navigation_allowed {
                            // There is a new URL
                            response.url = Some(url);
                        } else {
                            response.blocked_navigation_request = Some(url);
                        }
                    }
                }
                EmbedderMsg::WebViewOpened(new_webview_id) => {
                    self.web_views.insert(new_webview_id, WebView {});
                    self.event_queue
                        .push(EmbedderEvent::FocusWebView(new_webview_id));
                }
                EmbedderMsg::WebViewClosed(webview_id) => {
                    self.web_views.remove(&webview_id);
                    self.focused_webview_id = None;
                }
                EmbedderMsg::WebViewFocused(webview_id) => {
                    self.focused_webview_id = Some(webview_id);
                }
                EmbedderMsg::WebViewBlurred => {
                    self.focused_webview_id = None;
                }
                EmbedderMsg::ChangePageTitle(title) => {
                    response.title = title;
                }
                EmbedderMsg::NewFavicon(url) => {
                    response.favicon_url = Some(url.as_url().to_owned());
                }
                EmbedderMsg::LoadStart => {
                    response.loading = Some(true);
                }
                EmbedderMsg::LoadComplete => {
                    response.loading = Some(false);
                }
                EmbedderMsg::ReadyToPresent => {
                    response.present = Some(true);
                }
                EmbedderMsg::HistoryChanged(urls, position) => {
                    response.url = Some(urls[position].as_url().to_owned());
                    response.can_go_back = Some(position > 0);
                    response.can_go_forward = Some(position < (urls.len() - 1));
                }
                // TODO: this is where page up/down or shortcuts are handled
                // EmbedderMsg::Keyboard(key_event) => {}
                _others => {
                    println!("handle_servo_events: {:?}", _others);
                }
            }
        }

        response
    }

    pub fn push_event(&mut self, event: EmbedderEvent) {
        self.event_queue.push(event);
    }
}
