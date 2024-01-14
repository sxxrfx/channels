pub struct Channel<T> {
    queue: UnsafeCell<MaybeUninit<T>>,
    item_ready: AtomicBool,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            queue: UnsafeCell::new(MaybeUninit::uninit()),
            item_ready: AtomicBool::new(false),
        }
    }

    pub fn send(&self, message: T) {
        todo!()
    }

    pub fn receive(&self) -> T {
        todo!()
    }
}

unsafe impl<T> Sync for Channel<T> where T: Send {}
