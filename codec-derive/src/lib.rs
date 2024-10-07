use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

struct FieldInfo {
    ident: Ident,
    ty: syn::Type,
}

struct CodecStruct {
    struct_name: Ident,
    generics: syn::Generics,
    fields: Vec<FieldInfo>,
}

impl CodecStruct {
    fn parse(ast: &DeriveInput) -> Self {
        let data_struct = match &ast.data {
            Data::Struct(s) => s,
            _ => panic!("`Codec` can only be derived for structs"),
        };

        let named_fields = match &data_struct.fields {
            Fields::Named(named_fields) => named_fields,
            _ => panic!("`Codec` can only be derived for structs with named fields"),
        };

        let fields = named_fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().unwrap().clone();
                let ty = field.ty.clone();
                FieldInfo { ident, ty }
            })
            .collect();

        CodecStruct {
            struct_name: ast.ident.clone(),
            generics: ast.generics.clone(),
            fields,
        }
    }

    fn generate_impl(&self, sol_mode: bool) -> TokenStream {
        let struct_name = &self.struct_name;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let header_sizes = self.fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE
            }
        });

        let is_dynamic = self.fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC
            }
        });

        let encode_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {#sol_mode}>>::encode(&self.#ident, buffer, field_offset)?;
                field_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE);
            }
        });

        let decode_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                result.#ident = <#ty as Encoder<B, ALIGN, {#sol_mode}>>::decode(buffer, field_offset)?;
                field_offset += <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE;
            }
        });

        let partial_decode_fields = self.fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                let (offset, length) = <#ty as Encoder<B, ALIGN, {#sol_mode}>>::partial_decode(buffer, field_offset)?;
                field_offset += <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE;
                total_length += length;
            }
        });

        quote! {
            impl<B: ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, {#sol_mode}> for #struct_name #ty_generics #where_clause {
                const HEADER_SIZE: usize = 0 #( + #header_sizes)*;
                const IS_DYNAMIC: bool = false #( || #is_dynamic)*;

                fn encode(&self, buffer: &mut BytesMut, mut field_offset: usize) -> Result<(), CodecError> {
                    if Self::IS_DYNAMIC {
                        crate::encoder::write_u32_aligned::<B, ALIGN>(buffer, 0, 32 as u32);
                        field_offset += align_up::<ALIGN>(4);
                    }
                    #( #encode_fields )*
                    Ok(())
                }

                fn decode(buffer: &impl Buf, mut field_offset: usize) -> Result<Self, CodecError> {
                    let mut result = Self::default();
                    #( #decode_fields )*
                    Ok(result)
                }

                fn partial_decode(buffer: &impl Buf, mut field_offset: usize) -> Result<(usize, usize), CodecError> {
                    let mut total_length = 0;
                    #( #partial_decode_fields )*
                    Ok((field_offset, total_length))
                }
            }
        }
    }
}

impl ToTokens for CodecStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let impl_true = self.generate_impl(true);
        let impl_false = self.generate_impl(false);
        tokens.extend(quote! {
            #impl_true
            #impl_false
        });
    }
}

#[proc_macro_derive(Codec)]
pub fn codec_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let codec_struct = CodecStruct::parse(&ast);
    codec_struct.into_token_stream().into()
}
