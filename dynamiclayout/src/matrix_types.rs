
use std::ops::{Index, IndexMut};
use {OffsetType, LengthType, StrideType, Field, ArrayField, ArrayHelper, LayoutError};
use layout::ArrayFieldLayout;
use load::{FieldSpan, LayoutInfo};
//use {LayoutInfo, ArrayFieldLayout, MatrixArrayFieldLayout, LayoutDynamicField, AccessDynamicField,
//     FieldSpan, OffsetType, LengthType, LayoutArrayDynamicField, AccessArrayDynamicField};

#[derive(Default, Debug)]
pub struct MatrixArrayFieldLayout {
    offset: OffsetType,
    array_stride: StrideType,
    matrix_stride: StrideType,
}

macro_rules! make_matrix_type {
    ($matrix_type:ident [$column_count:expr][$row_count:expr] $($field:expr),+) => (
        #[repr(C, packed)]
        #[derive(Debug, Copy, Clone)]
        pub struct $matrix_type ([[f32; $row_count]; $column_count]);

        impl $matrix_type {
            pub fn new(data: [[f32; $row_count]; $column_count]) -> $matrix_type {
                $matrix_type(data)
            }

            // TODO: Make sure this actually does what it should
            #[allow(dead_code)]
            unsafe fn accessor_from_layout<'a, 'b>(layout: &'a <Self as Field>::Layout, bytes: *mut u8) -> <Self as Field<'b>>::Accessor {
                [
                    $( &mut *(layout.offset_ptr(bytes, $field) as *mut [f32; $row_count]) ),+
                ]
            }
        }

        impl Index<usize> for $matrix_type {
            type Output = [f32; $row_count];

            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }

        impl IndexMut<usize> for $matrix_type {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.0[index]
            }
        }

        impl<'a> Field<'a> for $matrix_type {
            type Layout = ArrayFieldLayout;
            type Accessor = [&'a mut [f32; $row_count]; $column_count];

            fn make_layout(layout_field: ::LayoutInfo) -> Result<Self::Layout, LayoutError> {
                if let LayoutInfo::ArrayField (offset, stride) = layout_field {
                    Ok(ArrayFieldLayout::new(offset, stride))
                } else {
                    Err(LayoutError)
                }
            }

            fn get_field_spans(layout: &Self::Layout) -> Box<Iterator<Item=FieldSpan>> {
                let offset = layout.offset();
                let stride = layout.stride();
                // TODO: 0..4 vs. 0..$column_count
                Box::new((0..$column_count).map(move |i| FieldSpan {
                    offset: (offset + stride * i) as OffsetType,
                    length: (::std::mem::size_of::<f32>() * $row_count) as LengthType,
                }))
            }

            unsafe fn make_accessor(layout: &Self::Layout, data: *mut u8) -> Self::Accessor {
                $matrix_type::accessor_from_layout(layout, data)
            }
        }


        impl<'a, L, A> ArrayField<'a, L, A> for $matrix_type
            where L: ArrayHelper<'a, Item=<Self as Field<'a>>::Layout>,
                A: ArrayHelper<'a, Item=<Self as Field<'a>>::Accessor> {
            type ArrayLayout = MatrixArrayFieldLayout;
            //type ArrayAccessor = Vec<<$matrix_type as Field<'a>>::Accessor>;
            type ArrayAccessor = A::ArrayType;

            fn make_layout(layout_field: LayoutInfo) -> Result<Self::ArrayLayout, LayoutError> {
                if let LayoutInfo::MatrixArrayField(offset, array_stride, matrix_stride) = layout_field {
                    Ok(MatrixArrayFieldLayout { offset: offset, array_stride: array_stride, matrix_stride: matrix_stride })
                } else {
                    Err(LayoutError)
                }
            }

            fn get_field_spans(layout: &Self::ArrayLayout) -> Box<Iterator<Item=FieldSpan>> {
                let offset = layout.offset;
                let array_stride = layout.array_stride;
                let matrix_stride = layout.matrix_stride;
                Box::new((0..L::len() as u16).flat_map(move |i| (0..$column_count).map(move |r| FieldSpan {
                    offset: (offset + array_stride * i + matrix_stride * r) as OffsetType,
                    length: ::std::mem::size_of::<f32>() as LengthType * $row_count as LengthType,
                })))
            }

            unsafe fn make_accessor(layout: &Self::ArrayLayout, data: *mut u8) -> Self::ArrayAccessor {
                let mut ah = A::uninitialized();
                {
                    let slice = ah.as_mut_slice();
                    // The pointer given to accessor_from_layout already has the offset calculated, therefore use 0 here
                    // TODO: Is it really?
                    let matrix_layout = ArrayFieldLayout::new(0, layout.matrix_stride);
                    for i in 0..A::len() {
                        let offset = (i as OffsetType) * layout.array_stride + layout.offset;
                        let accessor = $matrix_type::accessor_from_layout(&matrix_layout, data.offset(offset as isize));
                        let target: *mut A::Item = &mut slice[i];
                        // Use ptr::write to avoid calling drop on the (uninitialized) target memory
                        ::std::ptr::write(target, accessor);
                    }
                }
                ah.into_array()
            }
        }
    );
}

make_matrix_type!(Matrix2 [2][2] 0, 1);
make_matrix_type!(Matrix2x3 [2][3] 0, 1);
make_matrix_type!(Matrix2x4 [2][4] 0, 1);
make_matrix_type!(Matrix3x2 [3][2] 0, 1, 2);
make_matrix_type!(Matrix3 [3][3] 0, 1, 2);
make_matrix_type!(Matrix3x4 [3][4] 0, 1, 2);
make_matrix_type!(Matrix4x2 [4][2] 0, 1, 2, 3);
make_matrix_type!(Matrix4x3 [4][3] 0, 1, 2, 3);
make_matrix_type!(Matrix4 [4][4] 0, 1, 2, 3);
