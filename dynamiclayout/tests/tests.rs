
#![allow(dead_code)]

extern crate dynamiclayout;

#[macro_use]
extern crate dynamiclayout_derive;

use dynamiclayout::{DynamicLayout, Field, ArrayField, Data};
use dynamiclayout::helper::UnsafeArrayHelper;
use dynamiclayout::load::{LayoutInfo, FieldSpan};
use dynamiclayout::load::LayoutInfo::*;
use dynamiclayout::vector_types::{Vec2, Vec3, Vec4};
use dynamiclayout::matrix_types::{Matrix4, Matrix2x3};


#[test]
fn primitive_layout() {
    let layout_info = PrimitiveField(0);
    let layout = <f32 as Field>::make_layout(layout_info).unwrap();

    let span = <f32 as Field>::get_field_spans(&layout).next().unwrap();
    assert_eq!(span.offset, 0);
    assert_eq!(span.length, 4);

    let mut value = 1.23f32;
    let data = &mut value as *mut f32 as *mut u8;
    let accessor = unsafe { <f32 as Field>::make_accessor(&layout, data) };
    assert_eq!(*accessor, 1.23f32);
}

#[test]
fn primitive_array_layout() {
    type DummyLayoutArray<'a> = [<f32 as Field<'a>>::Layout; 3];
    type DummyLayoutItem<'a> = <f32 as Field<'a>>::Layout;
    type LayoutHelper<'a> = UnsafeArrayHelper<'a, DummyLayoutArray<'a>, DummyLayoutItem<'a>>;

    type DummyAccessorArray<'a> = [<f32 as Field<'a>>::Accessor; 3];
    type DummyAccessorItem<'a> = <f32 as Field<'a>>::Accessor;
    type AccessorHelper<'a> = UnsafeArrayHelper<'a, DummyAccessorArray<'a>, DummyAccessorItem<'a>>;

    let layout_info = ArrayField(0, 4);
    let layout = <f32 as ArrayField<LayoutHelper, AccessorHelper>>::make_layout(layout_info).unwrap();

    let spans: Vec<_> = <f32 as ArrayField<LayoutHelper, AccessorHelper>>::get_field_spans(&layout).collect();
    let expected: &[FieldSpan] = &[FieldSpan::new(0, 4), FieldSpan::new(4, 4), FieldSpan::new(8, 4)];
    assert!(spans == expected);

    let mut values = [1.23f32, 34.5f32, 5.67f32];
    let data = values[..].as_mut_ptr() as *mut u8;
    let accessor = unsafe { <f32 as ArrayField<LayoutHelper, AccessorHelper>>::make_accessor(&layout, data) };
    assert_eq!(accessor[0], 1.23f32);
    assert_eq!(accessor[1], 34.5f32);
    assert_eq!(accessor[2], 5.67f32);
}


#[repr(C, packed)]
#[derive(Debug, Copy, Clone, DynamicLayout)]
pub struct Foo {
    pub three: Vec3,
    pub one: f32,
    pub four: Vec4,
    pub two: Vec2,
    pub compound: Bar,
}

const FOO_SIZE: usize = 124;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, DynamicLayout)]
pub struct Bar {
    pub one: f32,
    pub four: Vec4,
    pub matrix: Matrix4,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, DynamicLayout)]
pub struct Qux {
    pub one: f32,
    pub four: Vec4,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, DynamicLayout)]
pub struct PrimitiveArray {
    pub first: i32,
    pub array: [i32; 8],
    pub last: i32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, DynamicLayout)]
pub struct StructArray {
    pub array: [Qux; 2],
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, DynamicLayout)]
pub struct MatrixArray {
    array: [Matrix2x3; 2]
}

const BAR_FIELDS: &'static [(&'static str, LayoutInfo<'static>)] = &[("one", PrimitiveField(40)),
                                                                      ("four", PrimitiveField(44)),
                                                                      ("matrix",
                                                                       ArrayField(60, 16))];
const BAR_LAYOUT: LayoutInfo<'static> = StructField(&BAR_FIELDS);

const FOO_FIELDS: &'static [(&'static str, LayoutInfo<'static>)] = &[("three", PrimitiveField(0)),
                                                                      ("one", PrimitiveField(12)),
                                                                      ("four", PrimitiveField(16)),
                                                                      ("two", PrimitiveField(32)),
                                                                      ("compound", BAR_LAYOUT)];

const P_A_FIELDS: &'static [(&'static str, LayoutInfo<'static>)] = &[("first", PrimitiveField(0)),
                                                                      ("array", ArrayField(4, 4)),
                                                                      ("last", PrimitiveField(36))];

// Note that the matrices in the array are interleaved!
const M_A_FIELDS: &'static [(&'static str, LayoutInfo<'static>)] = &[("array",
                                                                       MatrixArrayField(0, 12, 24))];

const QUX_FIELDS_0: &'static [(&'static str, LayoutInfo<'static>)] = &[("one", PrimitiveField(0)),
                                                                      ("four", PrimitiveField(4))];

const QUX_FIELDS_1: &'static [(&'static str, LayoutInfo<'static>)] = &[("one", PrimitiveField(20)),
                                                                      ("four", PrimitiveField(24))];

const QUX_LAYOUT_0: LayoutInfo<'static> = StructField(&QUX_FIELDS_0);
const QUX_LAYOUT_1: LayoutInfo<'static> = StructField(&QUX_FIELDS_1);
const S_A_FIELDS: &'static [(&'static str, LayoutInfo<'static>)] = &[("array",
                                   StructArrayField(&[&QUX_LAYOUT_0,
                                                      &QUX_LAYOUT_1]))];

fn make_foo_layout() -> <Foo as DynamicLayout<'static>>::Layout {
    Foo::load_layout(&FOO_FIELDS).unwrap()
}

fn make_primitive_array_layout() -> <PrimitiveArray as DynamicLayout<'static>>::Layout {
    PrimitiveArray::load_layout(&P_A_FIELDS).unwrap()
}

fn make_matrix_array_layout() -> <MatrixArray as DynamicLayout<'static>>::Layout {
    MatrixArray::load_layout(&M_A_FIELDS).unwrap()
}

fn make_struct_array_layout() -> <StructArray as DynamicLayout<'static>>::Layout {
    StructArray::load_layout(&S_A_FIELDS).unwrap()
}

fn new_foo() -> Foo {
    Foo {
        three: Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        one: 4.0,
        four: Vec4 {
            x: 5.0,
            y: 6.0,
            z: 7.0,
            w: 8.0,
        },
        two: Vec2 { x: 9.0, y: 10.0 },
        compound: Bar {
            one: 11.0,
            four: Vec4 {
                x: 12.0,
                y: 13.0,
                z: 14.0,
                w: 15.0,
            },
            matrix: Matrix4::new([[101.0, 102.0, 103.0, 104.0],
                                  [105.0, 106.0, 107.0, 108.0],
                                  [109.0, 110.0, 111.0, 112.0],
                                  [113.0, 114.0, 115.0, 116.0]]),
        },
    }
}

#[test]
fn one_to_one_mapping() {
    let layout = make_foo_layout();
    let foo = new_foo();
    let mut foo_source = foo.clone();

    {
        let mut data = unsafe { Data::from_anything(&mut foo_source) };

        let acc = layout.make_accessor(&mut data).unwrap();

        assert_eq!(foo.three.x, acc.three.x);
        assert_eq!(foo.three.y, acc.three.y);
        assert_eq!(foo.three.z, acc.three.z);

        assert_eq!(foo.one, *acc.one);

        assert_eq!(foo.four.x, acc.four.x);
        assert_eq!(foo.four.y, acc.four.y);
        assert_eq!(foo.four.z, acc.four.z);
        assert_eq!(foo.four.w, acc.four.w);

        assert_eq!(foo.two.x, acc.two.x);
        assert_eq!(foo.two.y, acc.two.y);

        assert_eq!(foo.compound.one, *acc.compound.one);
        assert_eq!(foo.compound.four.x, acc.compound.four.x);
        assert_eq!(foo.compound.four.y, acc.compound.four.y);
        assert_eq!(foo.compound.four.z, acc.compound.four.z);
        assert_eq!(foo.compound.four.w, acc.compound.four.w);

        assert_eq!(foo.compound.matrix[0][0], acc.compound.matrix[0][0]);
        assert_eq!(foo.compound.matrix[0][1], acc.compound.matrix[0][1]);
        assert_eq!(foo.compound.matrix[0][2], acc.compound.matrix[0][2]);
        assert_eq!(foo.compound.matrix[0][3], acc.compound.matrix[0][3]);
        assert_eq!(foo.compound.matrix[1][0], acc.compound.matrix[1][0]);
        assert_eq!(foo.compound.matrix[1][1], acc.compound.matrix[1][1]);
        assert_eq!(foo.compound.matrix[1][2], acc.compound.matrix[1][2]);
        assert_eq!(foo.compound.matrix[1][3], acc.compound.matrix[1][3]);
        assert_eq!(foo.compound.matrix[2][0], acc.compound.matrix[2][0]);
        assert_eq!(foo.compound.matrix[2][1], acc.compound.matrix[2][1]);
        assert_eq!(foo.compound.matrix[2][2], acc.compound.matrix[2][2]);
        assert_eq!(foo.compound.matrix[2][3], acc.compound.matrix[2][3]);
        assert_eq!(foo.compound.matrix[3][0], acc.compound.matrix[3][0]);
        assert_eq!(foo.compound.matrix[3][1], acc.compound.matrix[3][1]);
        assert_eq!(foo.compound.matrix[3][2], acc.compound.matrix[3][2]);
        assert_eq!(foo.compound.matrix[3][3], acc.compound.matrix[3][3]);

        acc.three.y = 999.0;
        acc.two[0] = 888.0;
        *acc.one = 777.0;
    }

    assert_eq!(foo_source.three.y, 999.0);
    assert_eq!(foo_source.two.x, 888.0);
    assert_eq!(foo_source.one, 777.0);
}

#[test]
fn vector_indexing() {
    let vec = Vec4::new(1.0, 2.0, 3.0, 4.0);

    assert_eq!(vec[0], 1.0);
    assert_eq!(vec[1], 2.0);
    assert_eq!(vec[2], 3.0);
    assert_eq!(vec[3], 4.0);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn vector_out_of_bounds() {
    let mut vec = Vec3::new(1.0, 2.0, 3.0);
    vec[4] = 4.0;
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, DynamicLayout)]
pub struct Matrix {
    matrix: Matrix4
}

const MATRIX: [[f32; 4]; 4] = [[0.0f32; 4]; 4];

fn matrix_bytes() -> [u8; 64] {
    unsafe { ::std::mem::transmute(MATRIX) }
}

fn matrix_layout() -> <Matrix as Field<'static>>::Layout {
    const LAYOUT: &'static [(&'static str, LayoutInfo<'static>)] = &[("matrix", ArrayField(0, 16))];
    Matrix::load_layout(&LAYOUT).unwrap()
}

#[test]
fn dynamic_matrix_indexing() {
    let layout = matrix_layout();
    let mut bytes = matrix_bytes();
    let mut data = unsafe { Data::from_anything(&mut bytes) };
    let acc = layout.make_accessor(&mut data).unwrap();
    assert_eq!(acc.matrix[0][0], 0.0);
    assert_eq!(acc.matrix[3][3], 0.0);
    acc.matrix[2][2] = 5.0;
}

#[test]
fn field_spans() {
    let layout = make_foo_layout();
    let spans: Vec<_> =
        <Foo as Field<'static>>::get_field_spans(&layout).collect();
    println!("{:?}", spans);
    let min = spans.iter().min_by_key(|span| span.offset).unwrap();
    assert_eq!(min.offset, 0);
    let max = spans.iter().max_by_key(|span| span.offset).unwrap();
    assert_eq!((max.offset + max.length) as usize, FOO_SIZE);
}

#[test]
fn primitive_array() {
    let layout = make_primitive_array_layout();
    let mut pa = PrimitiveArray {
        first: 11,
        array: [1, 2, 3, 4, 5, 6, 7, 8],
        last: 99,
    };
    let mut data = unsafe { Data::from_anything(&mut pa) };
    let mut acc = layout.make_accessor(&mut data).unwrap();

    assert_eq!(*acc.first, 11);
    assert_eq!(acc.array[0], 1);
    assert_eq!(acc.array[1], 2);
    assert_eq!(acc.array[2], 3);
    assert_eq!(acc.array[3], 4);
    assert_eq!(acc.array[4], 5);
    assert_eq!(acc.array[5], 6);
    assert_eq!(acc.array[6], 7);
    assert_eq!(acc.array[7], 8);
    assert_eq!(*acc.last, 99);

    acc.array[3] = 15;
    assert_eq!(acc.array[3], 15);
}

#[test]
fn matrix_array() {
    let layout = make_matrix_array_layout();
    let mut ma: [[f32; 3]; 4] = [[111.0, 112.0, 113.0],
                                 [211.0, 212.0, 213.0],
                                 [121.0, 122.0, 123.0],
                                 [221.0, 222.0, 223.0]];
    let mut data = unsafe { Data::from_anything(&mut ma) };
    let acc = layout.make_accessor(&mut data).unwrap();

    assert_eq!(acc.array[0][0][0], 111.0);
    assert_eq!(acc.array[0][0][1], 112.0);
    assert_eq!(acc.array[0][0][2], 113.0);
    assert_eq!(acc.array[0][1][0], 121.0);
    assert_eq!(acc.array[0][1][1], 122.0);
    assert_eq!(acc.array[0][1][2], 123.0);

    assert_eq!(acc.array[1][0][0], 211.0);
    assert_eq!(acc.array[1][0][1], 212.0);
    assert_eq!(acc.array[1][0][2], 213.0);
    assert_eq!(acc.array[1][1][0], 221.0);
    assert_eq!(acc.array[1][1][1], 222.0);
    assert_eq!(acc.array[1][1][2], 223.0);
}

#[test]
fn struct_array() {
    let layout = make_struct_array_layout();
    let mut sa = StructArray {
        array: [Qux {
            one: 0.0,
            four: Vec4 { x: 0.0, y: 1.0, z: 2.0, w: 3.0 },
        }, Qux {
            one: 1.0,
            four: Vec4 { x: 10.0, y: 11.0, z: 12.0, w: 13.0 },
        }]
    };
    let mut data = unsafe { Data::from_anything(&mut sa) };
    let acc = layout.make_accessor(&mut data).unwrap();
    assert_eq!(*acc.array[0].one, 0.0);
    assert_eq!(*acc.array[1].one, 1.0);
    assert_eq!(acc.array[0].four.x, 0.0);
    assert_eq!(acc.array[0].four.y, 1.0);
    assert_eq!(acc.array[0].four.z, 2.0);
    assert_eq!(acc.array[0].four.w, 3.0);
    assert_eq!(acc.array[1].four.x, 10.0);
    assert_eq!(acc.array[1].four.y, 11.0);
    assert_eq!(acc.array[1].four.z, 12.0);
    assert_eq!(acc.array[1].four.w, 13.0);
}
