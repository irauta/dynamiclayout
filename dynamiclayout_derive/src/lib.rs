
#![recursion_limit="200"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::{Tokens};
use syn::{Body, VariantData, Field, Ident, Ty, ConstExpr};

struct ArrayFieldInfo<'a> {index: usize, ty: &'a Ty, size: &'a ConstExpr, layout: Ident, accessor: Ident}

#[proc_macro_derive(DynamicLayout)]
pub fn derive_dynamiclayout(input: TokenStream) -> TokenStream {
    let input_string = input.to_string();
    let ast = syn::parse_macro_input(&input_string).unwrap();
    if let Body::Struct(VariantData::Struct(ref fields)) = ast.body {
        let output = make_types(&ast.ident, fields);
        output.parse().unwrap()
    } else {
        panic!("Only structs with named fields are supported")
    }
}

fn make_types(original_name: &Ident, fields: &Vec<Field>) -> Tokens {
    let mod_name = Ident::new(original_name.to_string().to_lowercase() + "_dynamiclayout_derive_mod");
    let array_fields = collect_array_fields(fields);
    let layout_struct = layout_struct(fields, &array_fields);
    let accessor_struct = accessor_struct(fields, &array_fields);
    let impl_dynamic_layout = impl_dynamic_layout();
    let impl_field = impl_field(fields, &array_fields);
    let impl_array_field = impl_array_field();
    let array_helpers = make_array_helpers(&array_fields);
    quote!{
        #[doc(hidden)]
        pub mod #mod_name {
            #![allow(unused_imports)]
            use super::*;
            use ::std::mem;
            use ::std::ops;
            extern crate dynamiclayout;
            use dynamiclayout::{DynamicLayout, LayoutError, AccessorError, Field, ArrayField, ArrayHelper, Data, OffsetType};
            use dynamiclayout::load::{LoadStructLayout, LayoutInfo, FieldSpan};
            use super::#original_name as OriginalType;

            pub struct GeneratedLayout {
                fields: GeneratedLayoutFields,
                len: OffsetType,
            }

            impl GeneratedLayout {
                pub fn make_accessor<'a>(&self, data: &'a mut Data) -> Result<GeneratedAccessor<'a>, AccessorError> {
                    <OriginalType as DynamicLayout<'a>>::make_accessor(self, data)
                }

                pub fn required_data_len(&self) -> usize {
                    self.len as usize
                }
            }

            #layout_struct

            #accessor_struct

            #impl_dynamic_layout

            #impl_field

            #impl_array_field

            #array_helpers
        }
    }
}

fn make_array_helpers(fields: &Vec<ArrayFieldInfo>) -> Tokens {
    let helpers: Vec<_> = fields.iter().map(|field| {
        let ty = field.ty;
        let size = field.size;
        let layout_helper_name = &field.layout;
        let accessor_helper_name = &field.accessor;
        quote!{
            pub struct #layout_helper_name<'a> (mem::ManuallyDrop<[<#ty as Field<'a>>::Layout; #size]>);
            unsafe impl<'a> ArrayHelper<'a> for #layout_helper_name<'a> {
                type Item = <#ty as Field<'a>>::Layout;
                type ArrayType = [<#ty as Field<'a>>::Layout; #size];

                fn len() -> usize { #size }

                unsafe fn uninitialized() -> Self  { #layout_helper_name(mem::ManuallyDrop::new(mem::uninitialized())) }

                fn as_mut_slice(&mut self) -> &mut [Self::Item]  { ops::DerefMut::deref_mut(&mut self.0) }

                fn array_as_slice(array: &Self::ArrayType) -> &[Self::Item] { &array[..] }

                fn into_array(self) -> Self::ArrayType  { mem::ManuallyDrop::into_inner(self.0) }
            }
            pub struct #accessor_helper_name<'a> (mem::ManuallyDrop<[<#ty as Field<'a>>::Accessor; #size]>);
            unsafe impl<'a> ArrayHelper<'a> for #accessor_helper_name<'a> {
                type Item = <#ty as Field<'a>>::Accessor;
                type ArrayType = [<#ty as Field<'a>>::Accessor; #size];

                fn len() -> usize { #size }

                unsafe fn uninitialized() -> Self  { #accessor_helper_name(mem::ManuallyDrop::new(mem::uninitialized())) }

                fn as_mut_slice(&mut self) -> &mut [Self::Item]  { ops::DerefMut::deref_mut(&mut self.0) }

                fn array_as_slice(array: &Self::ArrayType) -> &[Self::Item] { &array[..] }

                fn into_array(self) -> Self::ArrayType  { mem::ManuallyDrop::into_inner(self.0) }
            }
        }
    }).collect();
    quote!{
        #(#helpers)*
    }
}

fn collect_array_fields(fields: &Vec<Field>) -> Vec<ArrayFieldInfo> {
    fields.iter().enumerate().filter_map(|(i, f)| match f.ty {
        Ty::Array(ref ty, ref size) => Some(ArrayFieldInfo {index: i, ty: ty.as_ref(), size, layout: layout_array_helper_name(i), accessor: accessor_array_helper_name(i)}),
        _ => None
    }).collect()
}

fn layout_array_helper_name(i: usize) -> Ident {
    format!("LayoutArrayHelper{}", i).into()
}

fn accessor_array_helper_name(i: usize) -> Ident {
    format!("AccessorArrayHelper{}", i).into()
}

fn layout_struct(fields: &Vec<Field>, array_fields: &Vec<ArrayFieldInfo>) -> Tokens {
    let layout_fields = trait_fields(fields, array_fields).map(|(name, trait_tokens, is_array)| {
        if is_array {
            quote! { #name: #trait_tokens::ArrayLayout }
        } else {
            quote! { #name: #trait_tokens::Layout }
        }
    });
    quote! {
        pub struct GeneratedLayoutFields {
            #(#layout_fields),*
        }
    }
}

fn accessor_struct(fields: &Vec<Field>, array_fields: &Vec<ArrayFieldInfo>) -> Tokens {
    let accessor_fields = trait_fields_non_static(fields, array_fields).map(|(name, trait_tokens, is_array)| {
        if is_array {
            quote! { #name: #trait_tokens::ArrayAccessor }
        } else {
            quote! { #name: #trait_tokens::Accessor }
        }
    });
    quote! {
        pub struct GeneratedAccessor<'a> {
            #(pub #accessor_fields),*
        }
    }
}

fn impl_field(fields: &Vec<Field>, array_fields: &Vec<ArrayFieldInfo>) -> Tokens {
    let layout_fields = trait_fields(fields, array_fields).map(|(name, trait_tokens, _is_array)| {
        quote! { #name: #trait_tokens::make_layout(layout_info.get_field_layout(stringify!(#name)).ok_or(LayoutError)?)? }
    });
    let accessor_fields = trait_fields_non_static(fields, array_fields).map(|(name, trait_tokens, _is_array)| {
        quote! { #name: #trait_tokens::make_accessor(&layout.fields.#name, data) }
    });
    let field_spans = trait_fields(fields, array_fields).map(|(name, trait_tokens, _is_array)| {
        quote! { .chain(#trait_tokens::get_field_spans(&layout.fields.#name)) }
    });
    quote!{
        impl<'a> Field<'a> for OriginalType {
            type Layout = GeneratedLayout;
            type Accessor = GeneratedAccessor<'a>;

            fn make_layout(layout_field: LayoutInfo) -> Result<Self::Layout, LayoutError> {
                if let LayoutInfo::StructField(layout_info) = layout_field {
                    let layout_fields = GeneratedLayoutFields {
                        #(#layout_fields),*
                    };
                    let mut outer = GeneratedLayout {
                        len: 0,
                        fields: layout_fields
                    };
                    outer.len = <OriginalType as Field>::get_field_spans(&outer).map(|span| span.offset + span.length).max().unwrap_or(0);
                    Ok(outer)
                } else {
                    Err(LayoutError)
                }
            }

            unsafe fn make_accessor(layout: &Self::Layout, data: *mut u8) -> Self::Accessor {
                GeneratedAccessor {
                    #(#accessor_fields),*
                }
            }

            fn get_field_spans(layout: &Self::Layout) -> Box<Iterator<Item = FieldSpan>> {
                Box::new(
                    ::std::iter::empty()
                    #( #field_spans )*
                )
            }
        }
    }
}

fn impl_array_field() -> Tokens {
    quote!{
        impl<'a, L, A> ArrayField<'a, L, A> for OriginalType
            where L: ArrayHelper<'a, Item=<Self as Field<'a>>::Layout>,
                A: ArrayHelper<'a, Item=<Self as Field<'a>>::Accessor> + 'a {

            type ArrayLayout = L::ArrayType;
            type ArrayAccessor = A::ArrayType;

            fn make_layout(layout_field: LayoutInfo) -> Result<Self::ArrayLayout, LayoutError> {
                dynamiclayout::make_array_layout::<OriginalType, L>(layout_field)
            }

            unsafe fn make_accessor(layout: &Self::ArrayLayout, data: *mut u8) -> Self::ArrayAccessor {
                dynamiclayout::make_array_accessor::<OriginalType, L, A>(layout, data)
            }

            fn get_field_spans(layout: &Self::ArrayLayout) -> Box<Iterator<Item = FieldSpan>> {
                dynamiclayout::get_array_field_spans::<OriginalType, L>(layout)
            }
        }
    }
}

fn impl_dynamic_layout() -> Tokens {
    quote! {
        impl<'a> DynamicLayout<'a> for OriginalType {
            type Layout = GeneratedLayout;
            type Accessor = GeneratedAccessor<'a>;

            fn load_layout(layout_info: &LoadStructLayout) -> Result<GeneratedLayout, LayoutError> {
                <OriginalType as Field>::make_layout(LayoutInfo::StructField(layout_info))
            }

            fn make_accessor(layout: &GeneratedLayout, data: &'a mut Data) -> Result<GeneratedAccessor<'a>, AccessorError> {
                if data.len() < layout.required_data_len() {
                    return Err(AccessorError {
                        required_data_len: layout.required_data_len(),
                        data_len: data.len(),
                    });
                }
                unsafe {
                    Ok(<OriginalType as Field>::make_accessor(layout, data.as_ptr()))
                }
            }
        }
    }
}

fn trait_fields<'a>(fields: &'a Vec<Field>, array_fields: &'a Vec<ArrayFieldInfo>) -> Box<Iterator<Item = (Ident, Tokens, bool)> + 'a> {
    Box::new(fields.iter().enumerate().map(move |(i, field)| {
        let name = field.ident.clone().unwrap();
        let ty = &field.ty;
        match *ty {
            Ty::Array(ref inner_ty, ref _len) => {
                let array_field = array_fields.iter().find(|a| a.index == i).unwrap();
                let layout_helper = &array_field.layout;
                let accessor_helper = &array_field.accessor;
                (name, quote! { <#inner_ty as ArrayField<'static, #layout_helper<'static>, #accessor_helper<'static>>> }, true)
            },
            _ => (name, quote! { <#ty as Field<'static>> }, false)
        }
    }))
}

fn trait_fields_non_static<'a>(fields: &'a Vec<Field>, array_fields: &'a Vec<ArrayFieldInfo>) -> Box<Iterator<Item = (Ident, Tokens, bool)> + 'a> {
    Box::new(fields.iter().enumerate().map(move |(i, field)| {
        let name = field.ident.clone().unwrap();
        let ty = &field.ty;
        match *ty {
            Ty::Array(ref inner_ty, ref _len) => {
                let array_field = array_fields.iter().find(|a| a.index == i).unwrap();
                let layout_helper = &array_field.layout;
                let accessor_helper = &array_field.accessor;
                (name, quote! { <#inner_ty as ArrayField<'a, #layout_helper<'a>, #accessor_helper<'a>>> }, true)
            },
            _ => (name, quote! { <#ty as Field<'a>> }, false)
        }
    }))
}
