
use super::{OffsetType, StrideType, LengthType};

#[derive(Copy, Clone)]
pub enum LayoutInfo<'a> {
    PrimitiveField(OffsetType),
    ArrayField(OffsetType, StrideType),
    MatrixArrayField(OffsetType, StrideType, StrideType),
    StructField(&'a LoadStructLayout),
    StructArrayField(&'a [&'a LoadStructLayout]),
}


pub trait LoadStructLayout {
    fn get_field_layout(&self, field_name: &str) -> Option<LayoutInfo>;
}

impl<'a> LoadStructLayout for LayoutInfo<'a> {
    fn get_field_layout(&self, field_name: &str) -> Option<LayoutInfo> {
        match *self {
            LayoutInfo::StructField(ref inner) => inner.get_field_layout(field_name),
            _ => None,
        }
    }
}

impl<'a, F: AsRef<str>> LoadStructLayout for &'a [(F, LayoutInfo<'a>)] {
    fn get_field_layout(&self, field_name: &str) -> Option<LayoutInfo> {
        self.iter().find(|x| x.0.as_ref() == field_name).map(|x| x.1)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FieldSpan {
    pub offset: OffsetType,
    pub length: LengthType,
}

impl FieldSpan {
    pub fn new(offset: OffsetType, length: LengthType,) -> FieldSpan {
        FieldSpan {
            offset,
            length
        }
    }
}
