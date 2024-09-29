#![allow(dead_code)]

use std::alloc::alloc;
use std::alloc::dealloc;
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
                    ptr.drop_in_place();
                    self.len -= 1;
                    self.last = None;
                    return Some(data.data);
                };
                (*prev).next = None;
                dealloc(ptr as *mut u8, Layout::new::<Node<T>>());
                //ptr.drop_in_place();
                self.len -= 1;
                self.last = Some(prev);
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
                    ptr.drop_in_place();
                    self.head = None;
                    self.len -= 1;
                    return Some(data.data);
                };
                (*next).prev = None;
                dealloc(ptr as *mut u8, Layout::new::<Node<T>>());
                self.head = Some(next);
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
    pub fn new() -> Self {
        Self {
            head: None,
            last: None,
            len: 0,
        }
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

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        match self.last {
            Some(ptr) => {
                let mut current = ptr;
                loop {
                    if let Some(new) = unsafe { (*current).prev } {
                        unsafe {
                            current.drop_in_place();
                            dealloc(current as *mut u8, Layout::new::<Node<T>>())
                        };
                        current = new;
                    } else {
                        unsafe {
                            current.drop_in_place();
                            dealloc(current as *mut u8, Layout::new::<Node<T>>())
                        };
                        break;
                    }
                }
            }
            None => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_stuff() {
        let mut list = LinkedList::new();
        for val in 0..5 {
            list.insert(val);
        }
        assert_eq!(list.len(), 5);
        assert_eq!(list.pop(), Some(0));
        assert_eq!(list.pop_back(), Some(4));
        // for val in list {
        //     println!("{}", val)
        // }
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), None);
        assert_eq!(list.len(), 0);
    }
}
