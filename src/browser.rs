use servo::{BrowserId, embedder_traits::EmbedderMsg, compositing::windowing::EmbedderEvent};

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
    pub fn handle_servo_events(&mut self, events: Vec<(Option<BrowserId>, EmbedderMsg)>) {
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
                // TODO: this is where new page titles occur too
                // we will need to push these through to Qt
                // either pass in a Qt thread or maybe this moves into WebView?
                _others => {}
            }
        }
    }

    pub fn push_event(&mut self, event: EmbedderEvent) {
        self.event_queue.push(event);
    }
}
