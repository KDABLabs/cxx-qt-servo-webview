use cxx_qt::CxxQtThread;
use servo::embedder_traits::EventLoopWaker;

use crate::webview::qobject::ServoWebView;

pub(crate) struct QServoEventsLoopWaker {
    qt_loop: CxxQtThread<ServoWebView>,
}

impl QServoEventsLoopWaker {
    pub(crate) fn new(qt_loop: CxxQtThread<ServoWebView>) -> Self {
        Self { qt_loop }
    }
}

impl EventLoopWaker for QServoEventsLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(QServoEventsLoopWaker {
            qt_loop: self.qt_loop.clone(),
        })
    }

    fn wake(&self) {
        self.qt_loop
            .queue(|qobject| {
                qobject.handle_events();
            })
            .unwrap();
    }
}
