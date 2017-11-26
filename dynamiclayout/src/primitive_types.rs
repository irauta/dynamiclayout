
use {Field, ArrayField, LayoutError, ArrayHelper, LengthType, OffsetType};
use load::{LayoutInfo, FieldSpan};
use layout::{SimpleFieldLayout, ArrayFieldLayout};
use accessor::PrimitiveArrayAccessor;
use vector_types::*;

macro_rules! impl_primitive_type {
    ($primitive_type:ty) => (
        impl<'a> Field<'a> for $primitive_type {
            type Layout = SimpleFieldLayout;
            type Accessor = &'a mut $primitive_type;

            fn make_layout(layout_field: LayoutInfo) -> Result<Self::Layout, LayoutError> {
                if let LayoutInfo::PrimitiveField(offset) = layout_field {
                    Ok(SimpleFieldLayout::new(offset))
                } else {
                    Err(LayoutError)
                }
            }

            unsafe fn make_accessor(layout: &Self::Layout, data: *mut u8) -> &'a mut $primitive_type {
                let ptr = layout.offset_ptr(data);
                &mut *(ptr as *mut $primitive_type)
            }

            fn get_field_spans(layout: &Self::Layout) -> Box<Iterator<Item = FieldSpan>> {
                let span = FieldSpan {
                    offset: layout.offset(),
                    length: ::std::mem::size_of::<$primitive_type>() as LengthType,
                };
                Box::new(Some(span).into_iter())
            }
        }

        impl<'a, L, A> ArrayField<'a, L, A> for $primitive_type
            where L: ArrayHelper<'a, Item=<Self as Field<'a>>::Layout>,
                A: ArrayHelper<'a, Item=<Self as Field<'a>>::Accessor> {

            type ArrayLayout = ArrayFieldLayout;
            type ArrayAccessor = PrimitiveArrayAccessor<'a, $primitive_type>;

            fn make_layout(layout_field: LayoutInfo) -> Result<Self::ArrayLayout, LayoutError> {
                 if let LayoutInfo::ArrayField(offset, stride) = layout_field {
                    Ok(ArrayFieldLayout::new(offset, stride))
                } else {
                    Err(LayoutError)
                }
            }

            unsafe fn make_accessor(layout: &Self::ArrayLayout, data: *mut u8) -> Self::ArrayAccessor {
                let ptr = data.offset(layout.offset() as isize);
                PrimitiveArrayAccessor::new(ptr, layout.stride(), A::len())
            }

            fn get_field_spans(layout: &Self::ArrayLayout) -> Box<Iterator<Item = FieldSpan>> {
                let offset = layout.offset();
                let stride = layout.stride();
                Box::new((0..L::len() as OffsetType).map(move |i| FieldSpan {
                    offset: (offset + stride * i) as OffsetType,
                    length: ::std::mem::size_of::<$primitive_type>() as LengthType,
                }))
            }
        }

    )
}


impl_primitive_type!(f32);
impl_primitive_type!(i32);
impl_primitive_type!(u32);

impl_primitive_type!(Vec2);
impl_primitive_type!(IVec2);
impl_primitive_type!(UVec2);

impl_primitive_type!(Vec3);
impl_primitive_type!(IVec3);
impl_primitive_type!(UVec3);

impl_primitive_type!(Vec4);
impl_primitive_type!(IVec4);
impl_primitive_type!(UVec4);
