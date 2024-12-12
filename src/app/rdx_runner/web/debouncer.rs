use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;

struct Debouncer {
    timer: Rc<RefCell<Option<Timeout>>>,
    callback: Rc<RefCell<Box<dyn Fn()>>>,
    delay: u32,
}

impl Debouncer {
    fn new<F: Fn() + 'static>(callback: F, delay: u32) -> Self {
        Debouncer {
            timer: Rc::new(RefCell::new(None)),
            callback: Rc::new(RefCell::new(Box::new(callback))),
            delay,
        }
    }

    fn debounce(&self) {
        let timer = self.timer.clone();
        let callback = self.callback.clone();
        let delay = self.delay;

        if let Some(t) = timer.borrow_mut().take() {
            t.cancel();
        }

        let new_timer = Timeout::new(delay, move || {
            (*callback.borrow())();
        });

        *timer.borrow_mut() = Some(new_timer);
    }
}
