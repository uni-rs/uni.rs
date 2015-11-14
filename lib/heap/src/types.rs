use core::ptr;

pub struct UnsafeList<T> {
    head: Link<Node<T>>,
}

impl<T> UnsafeList<T> {
    pub unsafe fn new() -> Self {
        UnsafeList {
            head: Link::none(),
        }
    }

    pub unsafe fn push_front(&mut self, mut elem: Link<Node<T>>) {
        match self.head.as_mut() {
            None => {
                if let Some(ref mut node) = elem.as_mut() {
                    node.prev = Link::none();
                    node.next = Link::none();
                }
            },
            Some(ref mut node) => {
                if let Some(ref mut elem_node) = elem.as_mut() {
                    node.prev = elem.clone();

                    elem_node.next = self.head.clone();
                    elem_node.prev = Link::none();
                }
            }
        }

        self.head = elem;
    }

    pub unsafe fn pop(&mut self, mut elem: Link<Node<T>>) {
        if let Some(node) = elem.as_mut() {
            if let Some(prev) = node.prev.as_mut() {
                prev.next = node.next.clone();
            }

            if let Some(next) = node.next.as_mut() {
                next.prev = node.prev.clone();
            }
        } else {
            return;
        }

        if elem == self.head {
            if let Some(head) = self.head.as_mut() {
                self.head = head.next.clone();
            } else {
                self.head = Link::none();
            }
        }
    }
}

pub struct Node<T> {
    pub elem: T,
    pub prev: Link<Node<T>>,
    pub next: Link<Node<T>>,
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Self {
        Node {
            elem: elem,
            prev: Link::none(),
            next: Link::none(),
        }
    }
}

pub struct Link<T> {
    ptr: *mut T,
}

impl<T> Link<T> {
    pub fn some(ptr: *mut T) -> Self {
        Link {
            ptr: ptr,
        }
    }

    pub fn none() -> Self {
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
