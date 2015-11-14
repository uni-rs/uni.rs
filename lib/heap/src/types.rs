use core::ptr;

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
