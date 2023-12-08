use servo::{compositing::windowing::EmbedderEvent, embedder_traits::EmbedderMsg, BrowserId};

#[derive(Default)]
pub(crate) struct QServoBrowserResponse {
    pub(crate) present: bool,
    pub(crate) title: Option<String>,
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
                EmbedderMsg::ReadyToPresent => {
                    response.present = true;
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
