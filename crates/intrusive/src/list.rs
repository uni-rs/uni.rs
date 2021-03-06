//! Implementation of intrusive doubly linked list

use core::mem;

use core::ptr::Unique;
use core::marker::PhantomData;

use link::Link;

pub trait Node: Sized {
    fn prev(&self) -> &Link<Self>;
    fn next(&self) -> &Link<Self>;

    fn prev_mut(&mut self) -> &mut Link<Self>;
    fn next_mut(&mut self) -> &mut Link<Self>;
}

pub trait Owner<T> where T: Node {
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;

    fn take(self);

    unsafe fn from_raw_ptr(ptr: *mut T) -> Self;
}

pub struct List<T, U> where T: Owner<U>, U: Node {
    head: Link<U>,
    tail: Link<U>,
    _phantom1: PhantomData<T>,
}

impl<T, U> List<T, U> where T: Owner<U>, U: Node {
    pub const fn new() -> Self {
        List {
            head: Link::none(),
            tail: Link::none(),
            _phantom1: PhantomData,
        }
    }

    pub fn front(&self) -> Option<&U> {
        self.head.as_ref()
    }

    pub fn back(&self) -> Option<&U> {
        self.tail.as_ref()
    }

    pub fn front_mut(&mut self) -> Option<&mut U> {
        self.head.as_mut()
    }

    pub fn back_mut(&mut self) -> Option<&mut U> {
        self.tail.as_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.head == Link::none()
    }

    pub fn push_front(&mut self, mut elem: T) {
        match self.head.as_mut() {
            None => {
                *elem.get_mut().prev_mut() = Link::none();
                *elem.get_mut().next_mut() = Link::none();

                self.tail = Link::some(elem.get_mut());
            }
            Some(h) => {
                *elem.get_mut().prev_mut() = Link::none();
                *elem.get_mut().next_mut() = Link::some(h);

                *h.prev_mut() = Link::some(elem.get_mut());
            },
        }

        self.head = Link::some(elem.get_mut());

        elem.take();
    }

    pub fn push_back(&mut self, mut elem: T) {
        match self.tail.as_mut() {
            None => {
                *elem.get_mut().prev_mut() = Link::none();
                *elem.get_mut().next_mut() = Link::none();

                self.head = Link::some(elem.get_mut());
            }
            Some(t) => {
                *elem.get_mut().prev_mut() = Link::some(t);
                *elem.get_mut().next_mut() = Link::none();

                *t.next_mut() = Link::some(elem.get_mut());
            }
        }

        self.tail = Link::some(elem.get_mut());

        elem.take();
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().as_mut().and_then(|h| {
            match h.next_mut().as_mut() {
                None => self.tail = Link::none(),
                Some(n) => {
                    *n.prev_mut() = Link::none();

                    self.head = Link::some(n);
                }
            }

            Some(unsafe { T::from_raw_ptr(h) })
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().as_mut().and_then(|t| {
            match t.prev_mut().as_mut() {
                None => self.head = Link::none(),
                Some(p) => {
                    *p.next_mut() = Link::none();

                    self.tail = Link::some(p);
                }
            }

            Some(unsafe { T::from_raw_ptr(t) })
        })
    }

    pub fn cursor<'a>(&'a mut self) -> Cursor<'a, T, U> {
        Cursor {
            list: self,
            pos: Link::none(),
        }
    }
}

pub struct Cursor<'a, T, U> where T: Owner<U> + 'a, U: Node + 'a {
    list: &'a mut List<T, U>,
    pos: Link<U>,
}

impl<'a, T, U> Cursor<'a, T, U> where T: Owner<U> + 'a, U: Node + 'a {
    pub fn prev(&mut self) -> Option<&mut U> {
        match self.pos.take().as_mut() {
            None => {
                self.pos = self.list.tail.clone();
                None
            }
            Some(p) => {
                self.pos = p.prev().clone();
                Some(unsafe { mem::transmute(p) })
            }
        }
    }

    pub fn next(&mut self) -> Option<&mut U> {
        match self.pos.take().as_mut() {
            None => match self.list.head.as_mut() {
                // Empty list
                None => None,
                // We are at the head of the list
                Some(h) => {
                    self.pos = Link::some(h);
                    self.pos.as_mut()
                }
            },
            Some (p) => match p.next_mut().as_mut() {
                // End of the list
                None => {
                    self.pos = Link::none();
                    None
                }
                Some(n) => {
                    self.pos = Link::some(n);
                    self.pos.as_mut()
                }
            },
        }
    }

    pub fn prev_peek(&mut self) -> Option<&mut U> {
        self.pos.as_mut()
    }

    pub fn next_peek(&mut self) -> Option<&mut U> {
        match self.pos.as_mut() {
            None => self.list.front_mut(),
            Some(p) => p.next_mut().as_mut(),
        }
    }

    pub fn remove(&mut self) -> Option<T> {
        match self.pos.as_mut() {
            // Start of the list or empty
            None => self.list.pop_front(),
            Some(p) => match p.next_mut().take().as_mut() {
                // End of the list
                None => None,
                Some(n) => {
                    match n.next_mut().as_mut() {
                        None => self.list.tail = Link::some(p),
                        Some(next) => {
                            *next.prev_mut() = Link::some(p);
                            *p.next_mut() = Link::some(next);
                        }
                    }

                    Some(unsafe { T::from_raw_ptr(n) })
                }
            }
        }
    }
}

impl<T> Owner<T> for Unique<T> where T: Node {
    #[inline]
    fn get(&self) -> &T {
        unsafe { self.get() }
    }

    #[inline]
    fn get_mut(&mut self) -> &mut T {
        unsafe { self.get_mut() }
    }

    #[inline]
    fn take(self) {}

    unsafe fn from_raw_ptr(ptr: *mut T) -> Self {
        Unique::new(ptr)
    }
}

#[cfg(test)]
impl<T> Owner<T> for ::std::boxed::Box<T> where T: Node {
    #[inline]
    fn get(&self) -> &T {
        &**self
    }

    #[inline]
    fn get_mut(&mut self) -> &mut T {
        &mut **self
    }

    #[inline]
    fn take(self) {
        ::std::boxed::Box::into_raw(self);
    }

    unsafe fn from_raw_ptr(ptr: *mut T) -> Self {
        ::std::boxed::Box::from_raw(ptr)
    }
}

#[cfg(test)]
mod test {
    use core::ptr::Unique;

    use std::boxed::Box;

    use list::{List, Node};
    use link::Link;

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
    fn test_unique() {
        let one = Box::new(UsizeNode::new(1));
        let two = Box::new(UsizeNode::new(2));
        let three = Box::new(UsizeNode::new(3));

        let one_ptr = Box::into_raw(one);
        let two_ptr = Box::into_raw(two);
        let three_ptr = Box::into_raw(three);

        let one_unique = unsafe { Unique::new(one_ptr) };
        let two_unique = unsafe { Unique::new(two_ptr) };
        let three_unique = unsafe { Unique::new(three_ptr) };

        let mut list = List::<Unique<UsizeNode>, UsizeNode>::new();

        list.push_front(three_unique);
        list.push_front(two_unique);
        list.push_front(one_unique);

        unsafe {
            assert_eq!(list.pop_back().unwrap().get_mut().data, 3);
            assert_eq!(list.pop_back().unwrap().get_mut().data, 2);
            assert_eq!(list.pop_back().unwrap().get_mut().data, 1);
            assert!(list.pop_back().is_none());
        }

        // Cleanup
        unsafe {
            Box::from_raw(one_ptr);
            Box::from_raw(two_ptr);
            Box::from_raw(three_ptr);
        }
    }

    #[test]
    fn test_basic() {
        let mut list = List::<Box<UsizeNode>, UsizeNode>::new();

        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);

        assert!(list.is_empty());

        list.push_front(Box::new(UsizeNode::new(2)));

        assert!(!list.is_empty());
        assert_eq!(list.front(), list.back());

        list.push_front(Box::new(UsizeNode::new(1)));
        list.push_front(Box::new(UsizeNode::new(0)));

        assert_eq!(list.front().unwrap().data, 0);
        assert_eq!(list.back().unwrap().data, 2);

        list.push_back(Box::new(UsizeNode::new(3)));
        assert_eq!(list.back().unwrap().data, 3);

        list.push_back(Box::new(UsizeNode::new(4)));
        assert_eq!(list.back().unwrap().data, 4);

        assert!(!list.is_empty());

        assert_eq!(list.pop_front().unwrap().data, 0);
        assert_eq!(list.pop_front().unwrap().data, 1);
        assert_eq!(list.pop_front().unwrap().data, 2);
        assert_eq!(list.pop_back().unwrap().data, 4);
        assert_eq!(list.pop_back().unwrap().data, 3);
        assert_eq!(list.pop_back(), None);

        assert!(list.is_empty());

        list.push_back(Box::new(UsizeNode::new(5)));
        assert_eq!(list.pop_front().unwrap().data, 5);
        assert_eq!(list.pop_front(), None);

        assert!(list.is_empty());

        list.push_back(Box::new(UsizeNode::new(6)));
        list.push_back(Box::new(UsizeNode::new(7)));

        assert!(!list.is_empty());

        list.front_mut().unwrap().data = 6;
        list.back_mut().unwrap().data = 7;

        assert_eq!(list.front().unwrap().data, 6);
        assert_eq!(list.back().unwrap().data, 7);

        assert_eq!(list.pop_back().unwrap().data, 7);

        assert!(!list.is_empty());

        assert_eq!(list.pop_front().unwrap().data, 6);
        assert!(list.pop_front().is_none());
        assert!(list.pop_back().is_none());

        assert!(list.is_empty());
    }

    #[test]
    fn test_basic_cursor() {
        let mut list = List::<Box<UsizeNode>, UsizeNode>::new();

        {
            let mut cursor = list.cursor();

            assert!(cursor.next().is_none());
            assert!(cursor.prev().is_none());
            assert!(cursor.prev().is_none());
            assert!(cursor.next().is_none());
        }

        list.push_back(Box::new(UsizeNode::new(0)));
        list.push_back(Box::new(UsizeNode::new(1)));
        list.push_back(Box::new(UsizeNode::new(2)));
        list.push_back(Box::new(UsizeNode::new(3)));

        assert_eq!(list.front().unwrap().data, 0);
        assert_eq!(list.back().unwrap().data, 3);

        let mut cursor = list.cursor();

        assert!(cursor.prev_peek().is_none());
        assert_eq!(cursor.next_peek().unwrap().data, 0);

        assert_eq!(cursor.next().unwrap().data, 0);
        assert_eq!(cursor.next().unwrap().data, 1);
        assert_eq!(cursor.next().unwrap().data, 2);
        assert_eq!(cursor.next().unwrap().data, 3);
        assert!(cursor.next().is_none());
        assert_eq!(cursor.next().unwrap().data, 0);

        assert_eq!(cursor.prev().unwrap().data, 0);
        assert!(cursor.prev().is_none());
        assert_eq!(cursor.prev().unwrap().data, 3);
        assert_eq!(cursor.prev().unwrap().data, 2);
        assert_eq!(cursor.prev().unwrap().data, 1);
        assert_eq!(cursor.prev().unwrap().data, 0);
        assert!(cursor.prev().is_none());
    }

    #[test]
    fn test_cursor_remove() {
        let mut list = List::<Box<UsizeNode>, UsizeNode>::new();

        list.push_back(Box::new(UsizeNode::new(0)));
        list.push_back(Box::new(UsizeNode::new(1)));
        list.push_back(Box::new(UsizeNode::new(2)));
        list.push_back(Box::new(UsizeNode::new(3)));

        {
            let mut cursor = list.cursor();

            assert_eq!(cursor.remove().unwrap().data, 0);
            assert_eq!(cursor.next().unwrap().data, 1);
            assert_eq!(cursor.remove().unwrap().data, 2);
            assert_eq!(cursor.remove().unwrap().data, 3);
            assert!(cursor.remove().is_none());
        }

        assert_eq!(list.front().unwrap().data, list.back().unwrap().data);
        assert_eq!(list.cursor().remove().unwrap().data, 1);
        assert!(list.is_empty());
    }
}
