use servo::{compositing::windowing::EmbedderEvent, embedder_traits::EmbedderMsg, BrowserId};

#[derive(Default)]
pub(crate) struct QServoBrowserResponse {
    pub(crate) favicon_url: Option<url::Url>,
    pub(crate) present: Option<bool>,
    pub(crate) title: Option<String>,
    pub(crate) loading: Option<bool>,
}

#[derive(Default)]
pub(crate) struct QServoBrowser {
    browser_id: Option<BrowserId>,
    event_queue: Vec<EmbedderEvent>,
}

impl QServoBrowser {
    pub fn browser_id(&self) -> Option<BrowserId> {
        self.browser_id
    }

    pub fn get_events(&mut self) -> Vec<EmbedderEvent> {
        std::mem::take(&mut self.event_queue)
    }

    /// Returns true if the caller needs to manually present a new frame.
    ///
    /// TODO: does this move into the WebView?
    pub fn handle_servo_events(
        &mut self,
        events: Vec<(Option<BrowserId>, EmbedderMsg)>,
    ) -> QServoBrowserResponse {
        let mut response = QServoBrowserResponse::default();

        for (_browser_id, msg) in events {
            match msg {
                EmbedderMsg::BrowserCreated(new_browser_id) => {
                    if self.browser_id.is_some() {
                        panic!("Multiple top level browsing contexts not supported yet.");
                    }

                    self.browser_id = Some(new_browser_id);

                    self.event_queue
                        .push(EmbedderEvent::SelectBrowser(new_browser_id));
                }
                EmbedderMsg::ChangePageTitle(title) => {
                    response.title = title;
                }
                EmbedderMsg::NewFavicon(url) => {
                    response.favicon_url = Some(url.as_url().to_owned());
                }
                EmbedderMsg::LoadStart => {
                    response.loading = Some(true);
                },
                EmbedderMsg::LoadComplete => {
                    response.loading = Some(false);
                },
                EmbedderMsg::ReadyToPresent => {
                    response.present = Some(true);
                }
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
