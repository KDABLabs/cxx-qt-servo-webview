use servo::compositing::windowing::EmbedderMethods;
use servo::embedder_traits::EventLoopWaker;

// https://doc.servo.org/embedder_traits/trait.EventLoopWaker.html
pub(crate) struct QServoEmbedder {
    event_loop_waker: Box<dyn EventLoopWaker>,
}

impl QServoEmbedder {
    pub fn new(event_loop_waker: Box<dyn EventLoopWaker>) -> Self {
        Self { event_loop_waker }
    }
}

// https://doc.servo.org/compositing/windowing/trait.EmbedderMethods.html
impl EmbedderMethods for QServoEmbedder {
    fn create_event_loop_waker(&mut self) -> Box<dyn servo::embedder_traits::EventLoopWaker> {
        self.event_loop_waker.clone()
    }
}
