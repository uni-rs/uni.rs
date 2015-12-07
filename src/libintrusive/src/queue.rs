//! Implementation of an intrusive queue

use list::{List, Owner, Node};

pub struct Queue<T, U> where T: Owner<U>, U: Node {
    list: List<T, U>,
}

impl<T, U> Queue<T, U> where T: Owner<U>, U: Node {
    pub const fn new() -> Self {
        Queue {
            list: List::new(),
        }
    }

    pub fn enqueue(&mut self, elem: T) {
        self.list.push_back(elem);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.list.pop_front()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[inline]
    pub fn front(&self) -> Option<&U> {
        self.list.front()
    }

    #[inline]
    pub fn back(&self) -> Option<&U> {
        self.list.back()
    }

    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut U> {
        self.list.front_mut()
    }

    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut U> {
        self.list.back_mut()
    }
}

#[cfg(test)]
mod test {
    use std::boxed::Box;

    use link::Link;
    use list::Node;

    use super::Queue;

    #[derive(Debug)]
    struct UsizeNode {
        pub data: usize,
        prev: Link<UsizeNode>,
        next: Link<UsizeNode>,
    }

    impl UsizeNode {
        pub fn new(data: usize) -> Self {
            UsizeNode {
                data: data,
                prev: Link::none(),
                next: Link::none(),
            }
        }
    }

    impl Node for UsizeNode {
        fn prev(&self) -> &Link<Self> {
            &self.prev
        }

        fn next(&self) -> &Link<Self> {
            &self.next
        }

        fn prev_mut(&mut self) -> &mut Link<Self> {
            &mut self.prev
        }

        fn next_mut(&mut self) -> &mut Link<Self> {
            &mut self.next
        }
    }

    impl PartialEq for UsizeNode {
        fn eq(&self, rhs: &Self) -> bool { self.data == rhs.data }
    }

    #[test]
    fn test_queue() {
        let mut queue = Queue::<Box<UsizeNode>, UsizeNode>::new();

        assert!(queue.is_empty());

        queue.enqueue(Box::new(UsizeNode::new(1)));
        queue.enqueue(Box::new(UsizeNode::new(10)));
        queue.enqueue(Box::new(UsizeNode::new(2)));

        assert_eq!(queue.front().unwrap().data, 1);
        assert_eq!(queue.back().unwrap().data, 2);

        assert_eq!(queue.dequeue().unwrap().data, 1);

        queue.front_mut().unwrap().data = 1;

        queue.enqueue(Box::new(UsizeNode::new(3)));
        queue.enqueue(Box::new(UsizeNode::new(10)));

        queue.back_mut().unwrap().data = 4;

        assert_eq!(queue.dequeue().unwrap().data, 1);
        assert_eq!(queue.dequeue().unwrap().data, 2);
        assert!(!queue.is_empty());
        assert_eq!(queue.dequeue().unwrap().data, 3);
        assert_eq!(queue.dequeue().unwrap().data, 4);

        assert!(queue.dequeue().is_none());
        assert!(queue.is_empty());
    }
}
