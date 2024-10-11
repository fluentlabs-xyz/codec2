use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse2,
    parse_macro_input,
    token::Token,
    Data,
    DeriveInput,
    ExprLit,
    Field,
    Fields,
    Ident,
    Lit,
};

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

    fn encode_static(&self, sol_mode: bool) -> TokenStream {
        let encode_static_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                if !<#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC {
                    <#ty as Encoder<B, ALIGN, {#sol_mode}>>::encode(&self.#ident, &mut tmp, current_offset)?;
                    current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE);
                }
            }
        });

        quote! {
            #( #encode_static_fields )*
        }
    }

    fn generate_dynamic_fields_count(&self, sol_mode: bool) -> TokenStream {
        self.fields.iter().fold(quote!(0), |acc, field| {
            let ty = &field.ty;
            quote! {
                #acc + <#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC as usize
            }
        })
    }
    fn generate_dynamic_fields_header_size(&self, sol_mode: bool) -> TokenStream {
        self.fields.iter().fold(quote!(0), |acc, field| {
            let ty = &field.ty;
            quote! {
                #acc + (<#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC as usize  * <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE)
            }
        })
    }

    fn encode_dynamic(&self, sol_mode: bool) -> TokenStream {
        let encode_dynamic_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                if <#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC {
                    <#ty as Encoder<B, ALIGN, {#sol_mode}>>::encode(&self.#ident, &mut tmp, current_offset)?;
                    current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE);
                }
            }
        });

        quote! {
            #( #encode_dynamic_fields )*
        }
    }

    fn decode_static(&self, sol_mode: bool) -> TokenStream {
        let decode_static_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                if !<#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC {
                    let #ident = <#ty as Encoder<B, ALIGN, {#sol_mode}>>::decode(&mut tmp, current_offset)?;
                    current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE);
                }
            }
        });

        quote! {
            #( #decode_static_fields )*
        }
    }

    fn decode_dynamic(&self, sol_mode: bool) -> TokenStream {
        let decode_dynamic_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                if <#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC {
                    <#ty as Encoder<B, ALIGN, {#sol_mode}>>::decode(&mut tmp, current_offset)?;
                    current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE);
                }
            }
        });

        quote! {
            #( #decode_dynamic_fields )*
        }
    }

    fn generate_decode(&self, field: &Field, sol_mode: bool) -> TokenStream {
        let ident = &field.ident;
        let ty = &field.ty;
        quote! {
            let #ident = <#ty as Encoder<B, ALIGN, {#sol_mode}>>::decode(&mut tmp, current_offset)?;
            current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE);
        }
    }

    fn generate_partial_decode(&self, field: &Field, sol_mode: bool) -> TokenStream {
        let ty = &field.ty;
        quote! {
            let (offset, length) = <#ty as Encoder<B, ALIGN, {#sol_mode}>>::partial_decode(buffer, current_offset)?;
            current_offset += <#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE;
            total_length += length;
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

        let is_dynamic_expr = self.fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC
            }
        });

        let is_dynamic = quote! {
            false #( || #is_dynamic_expr)*
        };

        let encode_static_fields = self.encode_static(sol_mode);
        let encode_dynamic_fields = self.encode_dynamic(sol_mode);

        let decode_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                let #ident = if <#ty as Encoder<B, ALIGN, {#sol_mode}>>::IS_DYNAMIC {
                    <#ty as Encoder<B, ALIGN, {#sol_mode}>>::decode(&mut tmp, current_offset)?
                } else {
                    <#ty as Encoder<B, ALIGN, {#sol_mode}>>::decode(&mut tmp, current_offset)?
                };
                current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {#sol_mode}>>::HEADER_SIZE);
            }
        });

        let dynamic_fields_count = self.generate_dynamic_fields_count(sol_mode);
        let dynamic_fields_header_size = self.generate_dynamic_fields_header_size(sol_mode);

        let struct_initialization = self.fields.iter().map(|field| {
            let ident = &field.ident;
            quote! {
                #ident
            }
        });

        quote! {
                impl<B: ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, {#sol_mode}> for #struct_name #ty_generics #where_clause {
                    const HEADER_SIZE: usize = 0 #( + #header_sizes)*;
                    const IS_DYNAMIC: bool = #is_dynamic;

                    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
                        let aligned_offset = align_up::<ALIGN>(offset);
                        let mut current_offset = aligned_offset;
                        let mut tmp = BytesMut::new();

                        // Encode static fields
                        #encode_static_fields

                        // Encode dynamic fields
                        if #is_dynamic {
                            let dynamic_fields_header_size = #dynamic_fields_header_size;

                            if tmp.len() < current_offset + dynamic_fields_header_size {
                                tmp.resize(current_offset + dynamic_fields_header_size, 0);
                            }
                            #encode_dynamic_fields
                            // Write dynamic struct offset
                            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, 32);
                        }

                        buf.extend_from_slice(&tmp);

                        Ok(())
                    }

                    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
                        let mut current_offset = align_up::<ALIGN>(offset);

                        let mut tmp = if #is_dynamic {
                            &buf.chunk()[32..]
                        } else {
                            buf.chunk()
                        };

                        #( #decode_fields )*

                        Ok(#struct_name {
                            #( #struct_initialization ),*
                        })
                    }

                    fn partial_decode(buffer: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
                       Ok((0,0))
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
    quote! {
        #codec_struct
    }
    .into()
}

trait IsDynamic {
    const IS_DYNAMIC: bool;
}

impl<T> IsDynamic for Vec<T> {
    const IS_DYNAMIC: bool = true;
}
fn is_dynamic(field: &Field) -> bool {
    let ty = &field.ty;
    let is_dynamic_expr: TokenStream = quote! {
        <Vec<u32> as IsDynamic>::IS_DYNAMIC
    };

    let parsed = parse2::<ExprLit>(is_dynamic_expr);

    eprintln!(">>> parsed: {:?}", parsed.err());
    // .ok()
    // .and_then(|expr_lit| match expr_lit.lit {
    //     Lit::Bool(lit_bool) => {
    //         let val = lit_bool.value();
    //         eprintln!(">>> val: {}", val);
    //         Some(val)
    //     }
    //     _ => {
    //         eprintln!(">>> not bool");
    //         None
    //     }
    // })
    // .unwrap_or(false)
    false
}
