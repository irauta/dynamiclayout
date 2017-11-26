
use std::ops::{Index, IndexMut};

macro_rules! make_vector_type {
    ($vector_type:ident : $field_type:ty [$field_count:expr] $($field:ident),+) => (
        #[repr(C, packed)]
        #[derive(Debug, Copy, Clone)]
        pub struct $vector_type {
            $( pub $field: $field_type ),+
        }

        impl $vector_type {
            pub fn new( $( $field: $field_type ),+ ) -> $vector_type {
                $vector_type {
                    $( $field: $field ),+
                }
            }
        }

        impl Index<usize> for $vector_type {
            type Output = $field_type;

            fn index(&self, index: usize) -> &Self::Output {
                let array = unsafe {
                    &*(self as *const Self as *const [$field_type; $field_count])
                };
                &array[index]
            }
        }

        impl IndexMut<usize> for $vector_type {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                let array = unsafe {
                    &mut *(self as *mut Self as *mut [$field_type; $field_count])
                };
                &mut array[index]
            }
        }
    )
}

make_vector_type!(Vec2: f32 [2] x, y);
make_vector_type!(IVec2: i32 [2] x, y);
make_vector_type!(UVec2: u32 [2] x, y);

make_vector_type!(Vec3: f32 [3] x, y, z);
make_vector_type!(IVec3: i32 [3] x, y, z);
make_vector_type!(UVec3: u32 [3] x, y, z);

make_vector_type!(Vec4: f32 [4] x, y, z, w);
make_vector_type!(IVec4: i32 [4] x, y, z, w);
make_vector_type!(UVec4: u32 [4] x, y, z, w);
