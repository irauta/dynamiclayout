
pub mod load;
pub mod layout;
pub mod accessor;
pub mod vector_types;
pub mod matrix_types;
pub mod primitive_types;
pub mod helper;

use load::{LoadStructLayout, LayoutInfo, FieldSpan};

pub type OffsetType = u16;
pub type StrideType = u16;
pub type LengthType = u16;

#[derive(Debug)]
pub struct LayoutError;

#[derive(Debug)]
pub struct AccessorError {
    pub required_data_len: usize,
    pub data_len: usize,
}

pub trait DynamicLayout<'a> {
    type Layout;
    type Accessor: 'a;

    fn load_layout(layout_info: &LoadStructLayout) -> Result<Self::Layout, LayoutError>;

    fn make_accessor(layout: &Self::Layout, data: &'a mut Data) -> Result<Self::Accessor, AccessorError>;
}

pub trait Field<'a> {
    type Layout;
    type Accessor: 'a;

    fn make_layout(layout_field: LayoutInfo) -> Result<Self::Layout, LayoutError>;

    unsafe fn make_accessor(layout: &Self::Layout, data: *mut u8) -> Self::Accessor;

    fn get_field_spans(layout: &Self::Layout) -> Box<Iterator<Item = FieldSpan>>;
}

pub trait ArrayField<'a, L, A> : Field<'a>
    where L: ArrayHelper<'a, Item=<Self as Field<'a>>::Layout>,
        A: ArrayHelper<'a, Item=<Self as Field<'a>>::Accessor> {

    type ArrayLayout;
    type ArrayAccessor: 'a;

    fn make_layout(layout_field: LayoutInfo) -> Result<Self::ArrayLayout, LayoutError>;

    unsafe fn make_accessor(layout: &Self::ArrayLayout, data: *mut u8) -> Self::ArrayAccessor;

    fn get_field_spans(layout: &Self::ArrayLayout) -> Box<Iterator<Item = FieldSpan>>;
}

pub unsafe trait ArrayHelper<'a> {
    type Item;
    type ArrayType: 'a;

    fn len() -> usize;

    unsafe fn uninitialized() -> Self;

    fn as_mut_slice(&mut self) -> &mut [Self::Item];

    fn array_as_slice(array: &Self::ArrayType) -> &[Self::Item];

    fn into_array(self) -> Self::ArrayType;
}

pub struct Data<'a> {
    ptr: *mut u8,
    len: usize,
    _phantom: ::std::marker::PhantomData<&'a u8>
}

impl<'a> Data<'a> {
    // Unsafe because the u8 slice might not be properly aligned for 32-bit or 64-bit wide variable access.
    pub unsafe fn from_u8(slice: &mut [u8]) -> Data {
        Data {
            ptr: slice.as_mut_ptr(),
            len: slice.len(),
            _phantom: ::std::marker::PhantomData
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn from_anything<T: Sized>(x: &mut T) -> Data {
        Data {
            ptr: x as *mut T as *mut u8,
            len: ::std::mem::size_of::<T>(),
            _phantom: ::std::marker::PhantomData
        }
    }

    pub unsafe fn unsafe_clone(&self) -> Data {
        Data {
            ptr: self.ptr, len: self.len, _phantom: ::std::marker::PhantomData
        }
    }

    pub fn as_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
}

pub fn make_array_layout<'a, T, L>(layout_field: LayoutInfo) -> Result<L::ArrayType, LayoutError>
        where T: Field<'a>, L: ArrayHelper<'a, Item=<T as Field<'a>>::Layout> {
    if let LayoutInfo::StructArrayField(elements) = layout_field {
        let mut helper = unsafe { L::uninitialized() };
        {
            let slice = helper.as_mut_slice();
            if slice.len() != elements.len() {
                return Err(LayoutError);
            }
            for i in 0..slice.len() {
                let target = &mut slice[i];
                let layout = <T as Field<'a>>::make_layout(LayoutInfo::StructField(elements[i]))?;
                unsafe {
                    ::std::ptr::write(target, layout);
                }
            }
        }
        Ok(helper.into_array())
    } else {
        Err(LayoutError)
    }
}

pub unsafe fn make_array_accessor<'a, T, L, A>(layout: &L::ArrayType, data: *mut u8) -> A::ArrayType
        where T: Field<'a>, L: ArrayHelper<'a, Item=<T as Field<'a>>::Layout>,
        A: ArrayHelper<'a, Item=<T as Field<'a>>::Accessor> + 'a {
    let layout = <L as ArrayHelper>::array_as_slice(layout);
    let mut helper = A::uninitialized();
    {
        let slice = helper.as_mut_slice();
        if layout.len() != slice.len() {
            panic!("dynamiclayout::ArrayField has been misimplemented, layout and accessor lengths mismatch!");
        }
        for i in 0..slice.len() {
            let target = &mut slice[i];
            let accessor = <T as Field<'a>>::make_accessor(&layout[i], data);
            ::std::ptr::write(target, accessor);
        }
    }
    helper.into_array()
}

pub fn get_array_field_spans<'a, T, L>(layout: &L::ArrayType) -> Box<Iterator<Item = FieldSpan>>
        where T: Field<'a>, L: ArrayHelper<'a, Item=<T as Field<'a>>::Layout> {
    let layouts = <L as ArrayHelper<'a>>::array_as_slice(layout);
    let spans: Vec<FieldSpan> = layouts.iter().flat_map(|l| <T as Field<'a>>::get_field_spans(l)).collect();
    Box::new(spans.into_iter())
}
