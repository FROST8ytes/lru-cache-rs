use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct Node<T: Copy> {
    pub value: T,
    pub next: Option<Rc<RefCell<Node<T>>>>,
    pub prev: Option<Weak<RefCell<Node<T>>>>,
}

impl<T: Copy> Node<T> {
    pub fn new(value: T) -> Self {
        Node {
            value,
            next: None,
            prev: None,
        }
    }
}

impl<T: Copy> From<Node<T>> for Option<Rc<RefCell<Node<T>>>> {
    fn from(node: Node<T>) -> Self {
        Some(Rc::new(RefCell::new(node)))
    }
}

type NodePtr<T> = Rc<RefCell<Node<T>>>;

pub struct List<T: Copy> {
    head: Option<NodePtr<T>>,
    tail: Option<NodePtr<T>>,
    count: usize,
}

impl<T: Copy> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
            count: 0,
        }
    }

    pub fn push_front(&mut self, value: T) {
        let mut node = Node::new(value);

        match self.head.take() {
            None => {
                self.head = node.into();
                self.tail = self.head.clone();
            }
            Some(current_head) => {
                node.next = Some(current_head.clone());
                self.head = node.into();
                if let Some(h) = &self.head {
                    current_head.borrow_mut().prev = Some(Rc::downgrade(&h));
                }
            }
        };

        self.count += 1;
    }

    pub fn push_back(&mut self, value: T) {
        let mut node = Node::new(value);

        match self.tail.take() {
            None => {
                self.head = node.into();
                self.tail = self.head.clone();
            }
            Some(current_tail) => {
                node.prev = Some(Rc::downgrade(&current_tail));
                self.tail = node.into();
                current_tail.borrow_mut().next = self.tail.clone();
            }
        };

        self.count += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        match self.tail.take() {
            None => None,
            Some(tail) => {
                let mut tail = tail.borrow_mut();
                let prev = tail.prev.take();
                match prev {
                    None => {
                        self.head.take();
                    }
                    Some(prev) => {
                        let prev = prev.upgrade();
                        if let Some(prev) = prev {
                            prev.borrow_mut().next = None;
                            self.tail = Some(prev);
                        }
                    }
                };

                self.count -= 1;
                Some(tail.value)
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        match self.head.take() {
            None => None,
            Some(head) => {
                let mut head = head.borrow_mut();
                let next = head.next.take();
                match next {
                    None => {
                        self.tail.take();
                    }
                    Some(next) => {
                        next.borrow_mut().prev = None;
                        self.head = Some(next);
                    }
                };

                self.count -= 1;
                Some(head.value)
            }
        }
    }

    pub fn iter(&self) -> ListIterator<T> {
        ListIterator {
            current: self.head.clone(),
            current_back: self.tail.clone(),
        }
    }

    pub fn remove_node(&mut self, node: &mut NodePtr<T>) {
        let (prev, next) = {
            let mut node = node.borrow_mut();
            let prev = match node.prev.take() {
                None => None,
                Some(prev) => prev.upgrade(),
            };
            let next = node.next.take();
            (prev, next)
        };

        match (prev, next) {
            (None, None) => {
                self.head = None;
                self.tail = None;
            }
            (None, Some(next)) => {
                next.borrow_mut().prev = None;
                self.head.replace(next);
            }
            (Some(prev), None) => {
                prev.borrow_mut().next = None;
                self.tail.replace(prev);
            }
            (Some(prev), Some(next)) => {
                next.borrow_mut().prev.replace(Rc::downgrade(&prev));
                prev.borrow_mut().next.replace(next);
            }
        }
    }

    pub fn move_node_to_back(&mut self, mut node: NodePtr<T>) {
        self.remove_node(&mut node);
        self.push_node_back(node);
    }

    pub fn push_node_back(&mut self, node: NodePtr<T>) {
        match self.tail.take() {
            None => {
                self.head.replace(node);
                self.tail = self.head.clone();
            }
            Some(current_tail) => {
                node.borrow_mut().prev.replace(Rc::downgrade(&current_tail));
                self.tail.replace(node);
                current_tail.borrow_mut().next = self.tail.clone();
            }
        }
    }

    pub fn get_weak_tail(&self) -> Option<Weak<RefCell<Node<T>>>> {
        match &self.tail {
            None => None,
            Some(tail) => Some(Rc::downgrade(tail)),
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

pub struct ListIterator<T: Copy> {
    current: Option<NodePtr<T>>,
    current_back: Option<NodePtr<T>>,
}

impl<T: Copy> Iterator for ListIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take();
        if current.is_none() {
            return None;
        }

        let current = current.unwrap();
        let current = current.borrow();
        self.current = current.next.clone();
        Some(current.value)
    }
}

impl<T: Copy> DoubleEndedIterator for ListIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let current = self.current_back.take();
        if current.is_none() {
            return None;
        }

        let current = current.unwrap();
        let current = current.borrow();
        match &current.prev {
            None => Some(current.value),
            Some(prev) => {
                self.current_back = prev.upgrade();
                Some(current.value)
            }
        }
    }
}

impl<T: Copy> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_back() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works_builds_list() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.push_back(4);

        assert_eq!(list.pop_back(), Some(4));
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn works_builds_list_front() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        list.push_front(4);

        assert_eq!(list.pop_front(), Some(4));
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn works_builds_list_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        list.push_front(4);

        let mut idx: usize = 0;

        for (i, j) in list.iter().zip(list.iter().rev()) {
            println!("Iteration {}: {}, {}", idx, i, j);
            idx += 1;
        }

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
