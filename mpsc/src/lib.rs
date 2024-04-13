#![forbid(unsafe_code)]

use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt::Debug,
    rc::{Rc, Weak},
};

use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("channel is closed")]
pub struct SendError<T: Debug> {
    pub value: T,
}

pub struct Sender<T> {
    queue: Rc<RefCell<VecDeque<T>>>,
    blocked: Rc<RefCell<bool>>,
}

impl<T: Debug> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if self.is_closed() {
            return Err(SendError { value });
        }
        self.queue.borrow_mut().push_back(value);
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        if Rc::weak_count(&self.queue) == 0 {
            return true;
        }
        *self.blocked.borrow()
    }

    pub fn same_channel(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.queue, &other.queue)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            blocked: self.blocked.clone(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("channel is empty")]
    Empty,
    #[error("channel is closed")]
    Closed,
}

pub struct Receiver<T> {
    queue: Weak<RefCell<VecDeque<T>>>,
    blocked: Rc<RefCell<bool>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Result<T, ReceiveError> {
        if let Some(c) = self.queue.upgrade() {
            //if *self.blocked.borrow() {
            //    return Err(ReceiveError::Closed);
            //}
            if let Some(r) = c.borrow_mut().pop_front() {
                Ok(r)
            } else {
                if *self.blocked.borrow() {
                    return Err(ReceiveError::Closed);
                }
                Err(ReceiveError::Empty)
            }
        } else {
            Err(ReceiveError::Closed)
        }
    }

    pub fn close(&mut self) {
        *self.blocked.borrow_mut() = true
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.close();
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let refbuffer = Rc::new(RefCell::new(VecDeque::<T>::new()));
    let refblock = Rc::new(RefCell::new(false));
    let receiver = Receiver {
        queue: Rc::downgrade(&refbuffer),
        blocked: refblock.clone(),
    };
    let sender: Sender<T> = Sender {
        queue: Rc::clone(&refbuffer),
        blocked: refblock.clone(),
    };
    (sender, receiver)
}
