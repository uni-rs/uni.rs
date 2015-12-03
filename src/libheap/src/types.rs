use core::ptr;

pub struct UnsafeList<T> where T: Node<T> {
    head: Link<T>,
}

impl<T> UnsafeList<T> where T: Node<T> {
    pub const fn new() -> Self {
        UnsafeList {
            head: Link::none(),
        }
    }

    pub unsafe fn push_front(&mut self, mut elem: Link<T>) {
        match self.head.as_mut() {
            None => {
                if let Some(node) = elem.as_mut() {
                    *node.prev_mut() = Link::none();
                    *node.next_mut() = Link::none();
                }
            },
            Some(node) => {
                if let Some(elem_node) = elem.as_mut() {
                    *node.prev_mut() = elem.clone();

                    *elem_node.prev_mut() = Link::none();
                    *elem_node.next_mut() = self.head.clone();
                }
            }
        }

        self.head = elem;
    }

    pub unsafe fn pop_front(&mut self) -> Link<T> {
        if self.head == Link::none() {
            return Link::none();
        }

        let ret = self.head.clone();
        let head = self.head.clone();

        self.pop(head);

        ret
    }

    pub unsafe fn pop(&mut self, mut elem: Link<T>) {
        if let Some(node) = elem.as_mut() {
            if let Some(prev) = node.prev_mut().as_mut() {
                *prev.next_mut() = node.next().clone();
            }

            if let Some(next) = node.next_mut().as_mut() {
                *next.prev_mut() = node.prev().clone();
            }
        } else {
            return;
        }

        if elem == self.head {
            if let Some(head) = self.head.as_mut() {
                self.head = head.next().clone();
            } else {
                self.head = Link::none();
            }
        }
    }

    pub unsafe fn cursor<'a>(&'a mut self) -> UnsafeCursor<'a, T> {
        let head_dup = self.head.clone();

        UnsafeCursor {
            head: &mut self.head,
            current: head_dup,
        }
    }
}

pub struct UnsafeCursor<'a, T: Node<T> + 'a> {
    head: &'a mut Link<T>,
    current: Link<T>,
}

impl<'a, T: Node<T> + 'a> UnsafeCursor<'a, T> {
    pub unsafe fn remove(&mut self) -> Link<T> {
        if self.current.is_null() {
            return Link::none();
        }

        let mut res = self.current.clone();
        let update_head = *self.head == self.current;

        self.next();

        if let Some(node) = res.as_mut() {
            if let Some(prev) = node.prev_mut().as_mut() {
                *prev.next_mut() = node.next().clone();
            }

            if let Some(next) = node.next_mut().as_mut() {
                *next.prev_mut() = node.prev().clone();
            }

            *node.next_mut() = Link::none();
            *node.prev_mut() = Link::none();
        }

        if update_head {
            *self.head = self.current.clone();
        }

        res
    }

    pub unsafe fn next(&mut self) {
        if let Some(node) = self.current.as_mut() {
            self.current = node.next().clone();
        }
    }

    pub unsafe fn as_ref(&self) -> Option<&T> {
        self.current.as_ref()
    }
}

pub trait Node<T> where T: Node<T> + Sized {
    fn prev(&self) -> &Link<T>;
    fn next(&self) -> &Link<T>;

    fn prev_mut(&mut self) -> &mut Link<T>;
    fn next_mut(&mut self) -> &mut Link<T>;
}

pub struct DataNode<T> {
    elem: T,
    prev: Link<DataNode<T>>,
    next: Link<DataNode<T>>,
}

impl<T> DataNode<T> {
    pub fn new(elem: T) -> Self {
        DataNode {
            elem: elem,
            prev: Link::none(),
            next: Link::none(),
        }
    }

    pub fn data(&self) -> &T {
        &self.elem
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.elem
    }
}

impl<T> Node<DataNode<T>> for DataNode<T> {
    fn prev(&self) -> &Link<DataNode<T>> {
        &self.prev
    }

    fn next(&self) -> &Link<DataNode<T>> {
        &self.next
    }

    fn prev_mut(&mut self) -> &mut Link<DataNode<T>> {
        &mut self.prev
    }

    fn next_mut(&mut self) -> &mut Link<DataNode<T>> {
        &mut self.next
    }
}

pub struct PhantomNode {
    prev: Link<PhantomNode>,
    next: Link<PhantomNode>,
}

impl Node<PhantomNode> for PhantomNode {
    fn prev(&self) -> &Link<PhantomNode> {
        &self.prev
    }

    fn next(&self) -> &Link<PhantomNode> {
        &self.next
    }

    fn prev_mut(&mut self) -> &mut Link<PhantomNode> {
        &mut self.prev
    }

    fn next_mut(&mut self) -> &mut Link<PhantomNode> {
        &mut self.next
    }
}

pub struct Link<T> {
    ptr: *mut T,
}

impl<T> Link<T> {
    pub const fn some(ptr: *mut T) -> Self {
        Link {
            ptr: ptr,
        }
    }

    pub const fn none() -> Self {
        Link {
            ptr: ptr::null_mut(),
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr == ptr::null_mut()
    }

    pub unsafe fn as_ref<'a>(&self) -> Option<&'a T> {
        self.ptr.as_ref()
    }

    pub unsafe fn as_mut<'a>(&mut self) -> Option<&'a mut T> {
        self.ptr.as_mut()
    }
}

impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        Link {
            ptr: self.ptr,
        }
    }
}

impl<T> PartialEq<Link<T>> for Link<T> {
    fn eq(&self, rhs: &Link<T>) -> bool {
        self.ptr == rhs.ptr
    }
}
