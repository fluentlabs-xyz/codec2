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

    fn generate_impl_sol(&self) -> TokenStream {
        let struct_name = &self.struct_name;
        let (_impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let header_sizes = self.fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {true}>>::HEADER_SIZE
            }
        });

        let is_dynamic_expr = self.fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {true}>>::IS_DYNAMIC
            }
        });

        let is_dynamic = quote! {
            false #( || #is_dynamic_expr)*
        };

        let encode_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                if <#ty as Encoder<B, ALIGN, {true}>>::IS_DYNAMIC {
                    <#ty as Encoder<B, ALIGN, {true}>>::encode(&self.#ident, &mut tmp, current_offset)?;
                    current_offset += align_up::<ALIGN>(4);
                } else {
                    <#ty as Encoder<B, ALIGN, {true}>>::encode(&self.#ident, &mut tmp, current_offset)?;
                    current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {true}>>::HEADER_SIZE);
                }
            }
        });

        let decode_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                let #ident = <#ty as Encoder<B, ALIGN, {true}>>::decode(&mut tmp, current_offset)?;

                current_offset += if <#ty as Encoder<B, ALIGN, {true}>>::IS_DYNAMIC {
                    32
                } else {
                    align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {true}>>::HEADER_SIZE)
                };
            }
        });

        let aligned_header_size = self.fields.iter().map(|field| {
            let ty = &field.ty;
            let ts = quote! {
                <#ty as Encoder<B, ALIGN, {true}>>
            };
            quote! {
                if #ts ::IS_DYNAMIC {
                    32
                } else {
                    align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {true}>>::HEADER_SIZE)
                }
            }
        });

        let aligned_header_size = quote! {
            0 #( + #aligned_header_size)*
        };

        let struct_initialization = self.fields.iter().map(|field| {
            let ident = &field.ident;
            quote! {
                #ident
            }
        });

        quote! {
            impl<B: ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, {true}> for #struct_name #ty_generics #where_clause {
                const HEADER_SIZE: usize = 0 #( + #header_sizes)*;
                const IS_DYNAMIC: bool = #is_dynamic;

                fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {

                    let aligned_offset = align_up::<ALIGN>(offset);
                    let is_dynamic = <Self as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC;
                    let aligned_header_size = #aligned_header_size;
                    let mut dynamic_fields_count = 0;

                    // Write the dynamic struct offset if necessary
                    if is_dynamic {
                        let buf_len = buf.len();
                        let offset = if buf_len == 0 {
                            32
                        } else {
                            buf_len
                        };
                        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, offset as u32);
                    }

                    // Create a new BytesMut for temporary storage
                    let mut tmp = BytesMut::zeroed(aligned_header_size);
                    let mut current_offset = 0; // Start at 0 for tmp buffer

                    // Encode static fields
                    #( #encode_fields )*

                    buf.extend_from_slice(&tmp);

                    Ok(())
                }

                fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
                    let mut aligned_offset = align_up::<ALIGN>(offset);

                    let mut tmp = if #is_dynamic {
                        let offset = read_u32_aligned::<B, ALIGN>(&buf.chunk(), aligned_offset)? as usize;
                        &buf.chunk()[offset..]
                    } else {
                        &buf.chunk()[aligned_offset..]
                    };

                    let mut current_offset = 0;

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

    fn generate_impl_wasm(&self) -> TokenStream {
        let struct_name = &self.struct_name;
        let (_impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let header_sizes = self.fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {false}>>::HEADER_SIZE)
            }
        });

        let is_dynamic_expr = self.fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {false}>>::IS_DYNAMIC
            }
        });

        let is_dynamic = quote! {
            false #( || #is_dynamic_expr)*
        };

        let encode_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                <#ty as Encoder<B, ALIGN, {false}>>::encode(&self.#ident, buf, current_offset)?;
                current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {false}>>::HEADER_SIZE);
            }
        });

        let decode_fields = self.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                let #ident = <#ty as Encoder<B, ALIGN, {false}>>::decode(buf, current_offset)?;

                current_offset += align_up::<ALIGN>(<#ty as Encoder<B, ALIGN, {false}>>::HEADER_SIZE);
            }
        });

        let struct_initialization = self.fields.iter().map(|field| {
            let ident = &field.ident;
            quote! {
                #ident
            }
        });

        quote! {
            impl<B:ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, {false}> for #struct_name #ty_generics #where_clause {
                const HEADER_SIZE: usize = 0 #( + #header_sizes)*;
                const IS_DYNAMIC: bool = #is_dynamic;

                fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
                    let mut current_offset = align_up::<ALIGN>(offset);
                    let header_size = <Self as Encoder<B, ALIGN, { false }>>::HEADER_SIZE;

                    if buf.len() < current_offset + header_size {
                        buf.resize(current_offset + header_size, 0);
                    }

                    #( #encode_fields )*

                    Ok(())
                }

                fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
                    let mut current_offset = align_up::<ALIGN>(offset);

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
        let sol_mode = self.generate_impl_sol();
        let wasm_mode = self.generate_impl_wasm();
        tokens.extend(quote! {
            #sol_mode
            #wasm_mode
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
