use std::collections::{
    VecDeque,
    vec_deque::{Iter, IterMut},
};

pub struct CappedVec<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T> CappedVec<T> {
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    #[inline]
    pub fn data(&self) -> &VecDeque<T> {
        &self.data
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub fn first(&self) -> Option<&T> {
        self.data.front()
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.data.back()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.data.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.data.iter_mut()
    }

    #[inline]
    pub fn push_back(&mut self, value: T) {
        if self.data.len() == self.capacity {
            self.data.pop_front();
        }

        self.data.push_back(value);
    }

    #[inline]
    pub fn push_front(&mut self, value: T) {
        if self.data.len() == self.capacity {
            self.data.pop_back();
        }

        self.data.push_front(value);
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<T> {
        self.data.pop_back()
    }

    #[inline]
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;

        while self.data.len() > self.capacity {
            self.data.pop_front();
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }
}
