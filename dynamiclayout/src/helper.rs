
use std::marker::PhantomData;
use ArrayHelper;

pub struct UnsafeArrayHelper<'a, A: 'a, I> (::std::mem::ManuallyDrop<A>, PhantomData<I>, PhantomData<&'a A>);

unsafe impl<'a, A, I> ArrayHelper<'a> for UnsafeArrayHelper<'a, A, I> {
    type Item = I;
    type ArrayType = A;

    fn len() -> usize {
        unsafe { opaque_array_len::<A, I>() }
    }

    unsafe fn uninitialized() -> UnsafeArrayHelper<'a, A, I> {
        UnsafeArrayHelper(::std::mem::ManuallyDrop::new(::std::mem::uninitialized::<A>()), PhantomData, PhantomData)
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        let contents = ::std::ops::DerefMut::deref_mut(&mut self.0);
        unsafe {
            opaque_array_slice_mut::<A, I>(contents)
        }
    }

    fn array_as_slice(array: &Self::ArrayType) -> &[Self::Item] {
        unsafe {
            opaque_array_slice_mut::<Self::ArrayType, Self::Item>(array)
        }
    }

    fn into_array(self) -> A {
        ::std::mem::ManuallyDrop::into_inner(self.0)
    }
}

pub unsafe fn opaque_array_len<Array, Element>() -> usize {
    use ::std::mem::size_of;
    let size_single = size_of::<Element>();
    let size_array = size_of::<Array>();
    debug_assert!(size_single > 0);
    debug_assert!(size_array > 0);
    debug_assert!(size_array > size_single);
    let len = size_array / size_single;
    debug_assert_eq!(size_array, size_single * len);
    len
}

pub unsafe fn opaque_array_slice_mut<Array, Element>(array: &Array) -> &mut [Element] {
    let ptr = ::std::mem::transmute(array as * const Array);
    let len = opaque_array_len::<Array, Element>();
    ::std::slice::from_raw_parts_mut::<Element>(ptr, len)
}
