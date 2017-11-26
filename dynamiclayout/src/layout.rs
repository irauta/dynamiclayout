
use {OffsetType, StrideType};

#[derive(Default, Debug)]
pub struct SimpleFieldLayout {
    offset: OffsetType,
}

impl SimpleFieldLayout {
    pub fn new(offset: OffsetType) -> SimpleFieldLayout {
        SimpleFieldLayout { offset }
    }

    pub fn offset(&self) -> OffsetType {
        self.offset
    }

    pub unsafe fn offset_ptr(&self, ptr: *mut u8) -> *mut u8 {
        ptr.offset(self.offset as isize)
    }
}

#[derive(Default, Debug)]
pub struct ArrayFieldLayout {
    offset: OffsetType,
    stride: StrideType,
}

impl ArrayFieldLayout {
    pub fn new(offset: OffsetType, stride: StrideType) -> ArrayFieldLayout {
        ArrayFieldLayout { offset, stride }
    }

    pub fn offset(&self) -> OffsetType {
        self.offset
    }

    pub fn stride(&self) -> StrideType {
        self.stride
    }

    pub unsafe fn offset_ptr(&self, ptr: *mut u8, index: usize) -> *mut u8 {
        let total_offset: isize = self.offset as isize + self.stride as isize * index as isize;
        ptr.offset(total_offset)
    }
}

#[derive(Default, Debug)]
pub struct MatrixArrayFieldLayout {
    offset: OffsetType,
    array_stride: StrideType,
    matrix_stride: StrideType,
}

impl MatrixArrayFieldLayout {
    pub fn new(offset: OffsetType,
               array_stride: StrideType,
               matrix_stride: StrideType) -> MatrixArrayFieldLayout {
        MatrixArrayFieldLayout { offset, array_stride, matrix_stride }
    }
}
