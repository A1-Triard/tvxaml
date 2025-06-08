use either::{Either, Left, Right};
use std::mem::replace;

pub struct EventHandler<T>(Either<(), T>);

impl<T> EventHandler<T> {
    pub fn new(handler: T) -> Self {
        EventHandler(Right(handler))
    }

    pub fn set(&mut self, handler: T) {
        self.0 = Right(handler);
    }

    pub fn begin_invoke(&mut self) -> T {
        let handler = replace(&mut self.0, Left(()));
        handler.right().unwrap()
    }

    pub fn end_invoke(&mut self, handler: T) {
        if self.0.is_left() {
            self.0 = Right(handler);
        }
    }
}

impl<T: Default> Default for EventHandler<T> {
    fn default() -> Self {
        EventHandler::new(T::default())
    }
}
