#![forbid(unsafe_code)]

use std::rc::Rc;

////////////////////////////////////////////////////////////////////////////////

pub struct PRef<T> {
    node: Rc<Node<T>>,
}

impl<T> std::ops::Deref for PRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node.value
    }
}

impl<T> Clone for PRef<T> {
    fn clone(&self) -> Self {
        Self {
            node: Rc::clone(&self.node),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct PSIterator<T> {
    stack: PStack<T>,
}

impl<T> Iterator for PSIterator<T> {
    type Item = PRef<T>;
    fn next(&mut self) -> Option<PRef<T>> {
        if let Some((e, s)) = self.stack.pop() {
            self.stack = s;
            return Some(e);
        }
        None
    }
}
pub struct Node<T> {
    value: T,
    prev_node: Option<PRef<T>>,
}

pub struct PStack<T> {
    node: Option<PRef<T>>,
    l: usize,
}

impl<T> Default for PStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for PStack<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            l: self.l,
        }
    }
}

impl<T> PStack<T> {
    pub fn new() -> Self {
        Self { node: None, l: 0 }
    }

    pub fn push(&self, value: T) -> Self {
        Self {
            node: Some(PRef {
                node: Rc::new(Node {
                    value,
                    prev_node: self.node.clone(),
                }),
            }),
            l: self.l + 1,
        }
    }

    pub fn pop(&self) -> Option<(PRef<T>, Self)> {
        if let Some(n) = self.node.clone() {
            return Some((
                n,
                Self {
                    node: self.node.clone().unwrap().node.prev_node.clone(),
                    l: self.l - 1,
                },
            ));
        }
        None
    }

    pub fn len(&self) -> usize {
        self.l
    }

    pub fn is_empty(&self) -> bool {
        self.node.is_none()
    }

    pub fn iter(&self) -> impl Iterator<Item = PRef<T>> {
        PSIterator {
            stack: self.clone(),
        }
    }
}
