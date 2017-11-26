
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use StrideType;

pub struct PrimitiveArrayAccessor<'a, T: 'a> {
    bytes: *mut u8,
    stride: StrideType,
    len: usize,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: 'a> PrimitiveArrayAccessor<'a, T> {
    pub unsafe fn new(bytes: *mut u8, stride: StrideType, len: usize) -> PrimitiveArrayAccessor<'a, T> {
        PrimitiveArrayAccessor {
            bytes,
            stride,
            len,
            phantom: PhantomData
        }
    }

    fn index(&self, index: usize) -> *mut T {
        if index >= self.len {
            panic!("PrimitiveArrayAccessor index out of bounds: the len is {} but the index is {}",
                   self.len,
                   index);
        }
        unsafe { self.bytes.offset(index as isize * self.stride as isize) as *mut T }
    }
}

impl<'a, T: 'a> Index<usize> for PrimitiveArrayAccessor<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.index(index) }
    }
}

impl<'a, T: 'a> IndexMut<usize> for PrimitiveArrayAccessor<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.index(index) }
    }
}
