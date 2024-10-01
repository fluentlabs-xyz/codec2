use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, Fields, Ident};

fn impl_derive_codec(ast: &syn::DeriveInput) -> TokenStream {
    let data_struct = match &ast.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("only structs are supported"),
    };
    let named_fields = match &data_struct.fields {
        Fields::Named(named_fields) => named_fields,
        _ => panic!("only named fields are supported"),
    };

    let struct_name = &ast.ident;
    let (_impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    let generate_impl = |sol_mode: bool| {
        let header_sizes = named_fields.named.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE
            }
        });

        let encode_types = named_fields.named.iter().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {#sol_mode}>>::encode(&self.#ident, buffer, field_offset)?;
                field_offset += <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE;
            }
        });

        let decode_types = named_fields.named.iter().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            quote! {
                result.#ident = <#ty as Encoder<B, ALIGN, {#sol_mode}>>::decode(buffer, field_offset)?;
                field_offset += <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE;
            }
        });

        let partial_decode_types = named_fields.named.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                let (offset, length) = <#ty as Encoder<B, ALIGN, {#sol_mode}>>::partial_decode(buffer, field_offset)?;
                field_offset += <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE;
                total_length += length;
            }
        });

        quote! {
            impl<B: ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, {#sol_mode}> for #struct_name #type_generics #where_clause {
                const HEADER_SIZE: usize = 0 #( + #header_sizes)*;

                fn encode(&self, buffer: &mut BytesMut, mut field_offset: usize) -> Result<(), CodecError> {
                    #( #encode_types )*
                    Ok(())
                }

                fn decode(buffer: &impl Buf, mut field_offset: usize) -> Result<Self, CodecError> {
                    let mut result = Self::default();
                    #( #decode_types )*
                    Ok(result)
                }

                fn partial_decode(buffer: &impl Buf, mut field_offset: usize) -> Result<(usize, usize), CodecError> {
                    let mut total_length = 0;
                    #( #partial_decode_types )*
                    Ok((field_offset, total_length))
                }
            }
        }
    };

    let impl_true = generate_impl(true);
    let impl_false = generate_impl(false);

    let output = quote! {
        #impl_true
        #impl_false
    };

    TokenStream::from(output)
}

#[proc_macro_derive(Codec)]
pub fn codec_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_derive_codec(&ast)
}
