use servo::embedder_traits::EventLoopWaker;

#[derive(Default)]
pub(crate) struct QServoEventsLoopWaker;

impl EventLoopWaker for QServoEventsLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(QServoEventsLoopWaker {})
    }

    // TODO: do we need to wake Qt here?
    fn wake(&self) {
        // probably need to trigger a QQuickItem::update ?
        println!("wake!");
    }
}
