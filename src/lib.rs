#![allow(dead_code)]

use std::alloc::alloc;
use std::alloc::Layout;

use std::iter::IntoIterator;
use std::iter::Iterator;
//this is currently pointer soup rn, have to figure out a way to add and keep track of all the nodes

pub struct LinkedList<T> {
    head: Option<*mut Node<T>>,
    last: Option<*mut Node<T>>,
    len: usize,
}

pub struct Node<T> {
    prev: Option<*mut Node<T>>,
    next: Option<*mut Node<T>>,
    data: T,
}

impl<T> LinkedList<T> {
    //allocate a new node and link it to the last element in the list
    unsafe fn __alloc_new_node_tail(&mut self, data: T) {
        let allocated = alloc(Layout::new::<Node<T>>()) as *mut Node<T>;
        let data = Node {
            prev: self.last,
            next: None,
            data,
        };
        allocated.write(data);
        match self.last {
            Some(ptr) => {
                (*ptr).next = Some(allocated);
                self.last = Some(allocated);
            }
            None => {
                self.head = Some(allocated);
                self.last = Some(allocated)
            }
        }
        self.len += 1
    }

    unsafe fn __alloc_new_node_head(&mut self, data: T) {
        let allocated = alloc(Layout::new::<Node<T>>()) as *mut Node<T>;
        let data = Node {
            prev: None,
            next: self.head,
            data,
        };
        allocated.write(data);
        match self.head {
            Some(ptr) => {
                (*ptr).prev = Some(allocated);
                self.head = Some(allocated)
            }
            None => {
                self.head = Some(allocated);
                self.last = Some(allocated)
            }
        }
        self.len += 1;
    }

    unsafe fn __remove_last(&mut self) -> Option<T> {
        //remember to deallocate everything
        match self.last {
            Some(ptr) => {
                let data = ptr.read();
                let prev = if let Some(prev_ptr) = data.prev {
                    prev_ptr
                } else {
                    unreachable!("What have I done?")
                };
                (*prev).next = None;
                ptr.drop_in_place();
                self.len -= 1;
                Some(data.data)
            }
            None => None,
        }
    }

    unsafe fn __remove_head(&mut self) -> Option<T> {
        match self.head {
            Some(ptr) => {
                let data = ptr.read();
                let next = if let Some(next_ptr) = data.next {
                    next_ptr
                } else {
                    unreachable!("Again I ask, what have I done?")
                };
                (*next).prev = None;
                ptr.drop_in_place();
                self.len -= 1;
                Some(data.data)
            }
            None => None,
        }
    }
    //data in current node, ptr to the next node
    fn get_next(&self, current: *mut Node<T>) -> (Option<*mut Node<T>>, T) {
        let data = unsafe { current.read() };
        let next = data.next;
        let data = data.data;
        unsafe { current.drop_in_place() };
        (next, data)
    }

    pub fn insert(&mut self, data: T) {
        unsafe { self.__alloc_new_node_tail(data) }
    }

    pub fn insert_front(&mut self, data: T) {
        unsafe { self.__alloc_new_node_head(data) }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.__remove_head() }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe { self.__remove_last() }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct LinkedListIterator<T> {
    internal: LinkedList<T>,
    current: Option<*mut Node<T>>,
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = LinkedListIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        let head = self.head.clone();
        LinkedListIterator {
            internal: self,
            current: head,
        }
    }
}

impl<T> Iterator for LinkedListIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(ptr) => {
                let (next, data) = self.internal.get_next(ptr);
                self.current = next;
                Some(data)
            }
            None => None,
        }
    }
}
