use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::quote;
use syn::{Ident, ItemEnum, Variant};

#[proc_macro_attribute]
pub fn generate_column_impls(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let output = column_impls(&TokenStream::from(attr), input);
    proc_macro::TokenStream::from(output)
}

fn column_impls(_attr: &TokenStream, input: proc_macro::TokenStream) -> TokenStream {
    let mut enumeration: ItemEnum = syn::parse(input).expect("Failed to parse input");
    let enum_ident = enumeration.ident.clone();
    let variants: Vec<Params> = enumeration
        .variants
        .iter_mut()
        .map(|var| get_params(var).expect("No parameter attributes present"))
        .collect();

    let (holder_vars, fn_vars): (TokenStream, TokenStream) = variants
        .iter()
        .map(|var| {
            let ident = var.ident.clone();
            let typ = var.typ.clone();
            let size = var.size.clone();

            let data_holder_variant = quote! {
                #ident(#typ),
            };

            let size_fn_variant = quote! {
                #enum_ident::#ident => #size,
            };

            (data_holder_variant, size_fn_variant)
        })
        .unzip();

    let holder_enum = quote! {
        #[derive(PartialEq, Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
        pub enum Data {
            #holder_vars
        }
    };
    let impl_enum = quote! {
        impl #enum_ident {
            pub const fn size(&self) -> usize {
                match self {
                    #fn_vars
                }
            }
        }
    };

    let attrs = quote! {
        #[derive(
            PartialEq,
            Copy,
            Clone,
            Eq,
            Debug,
            BorshSerialize,
            BorshDeserialize,
            Serialize,
            Deserialize,
            TryFromPrimitive,
            IntoPrimitive,
        )]
        #[repr(u8)]
    };

    let column_trait_implementations = variants.iter().map(|key| {
        let key_ident = key.ident.clone();
        let key_type = key.typ.clone();
        let key_size = key.size.clone();

        variants.iter().map(|value| {
            let value_ident = value.ident.clone();
            let value_type = value.typ.clone();
            let value_size = value.size.clone();
            quote!{
                impl<'a> ColumnTrait for RBTree<'a, #key_type, #value_type, #key_size, #value_size> {
                    fn get_key(&self, value: Data) -> Option<Data> {
                        None
                    }

                    fn get_value(&self, key: Data) -> Option<Data> {
                        if let Data::#key_ident(unwrapped_key) = key {
                            self.get(&unwrapped_key).map(|val| Data::#value_ident(val))
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn set(&mut self, key: Data, value: Data) -> Option<Data> {
                        if let (Data::#key_ident(unwrapped_key), Data::#value_ident(unwrapped_value)) = (key, value) {
                            self.insert(unwrapped_key, unwrapped_value)
                                .expect("Unexpected RBTree error")
                                .map(|old_value| Data::#value_ident(old_value))
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn delete_by_key(&mut self, key: Data) -> bool {
                        if let Data::#key_ident(unwrapped_key) = key {
                            self.delete(&unwrapped_key)
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn delete_by_value(&mut self, value: Data) -> bool {
                        panic!("It is impossible to delete by value in RBTree");
                    }
                }
            }
        }).collect::<TokenStream>()
    }).collect::<TokenStream>();

    let (slice_init_variants, slice_from_variants): (TokenStream, TokenStream)  = variants.iter().map(|key| {
        let key_ident = key.ident.clone();
        let key_type = key.typ.clone();
        let key_size = key.size.clone();

        variants.iter().map(|value| {
            let value_ident = value.ident.clone();
            let value_type = value.typ.clone();
            let value_size = value.size.clone();

            let init_variant = quote! {
                (DataType::#key_ident, DataType::#value_ident) => RBTree::<#key_type, #value_type, #key_size, #value_size>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn ColumnTrait>)
                    .map_err(|e| Error::from(e)),
            };

            let from_variant = quote! {
                (DataType::#key_ident, DataType::#value_ident) => RBTree::<#key_type, #value_type, #key_size, #value_size>::from_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn ColumnTrait>)
                    .map_err(|e| Error::from(e)),
            };

            (init_variant, from_variant)
        }).unzip::<TokenStream, TokenStream, TokenStream, TokenStream>()
    }).unzip();

    let from_slice_fn = quote! {
        pub fn from_column_slice<'a,'b: 'a>(
            pk_type: DataType,
            val_type: DataType,
            is_secondary_key: bool,
            slice: &'b mut [u8],
        ) -> Result<Box<dyn ColumnTrait + 'a>, Error> {
            if is_secondary_key {
                unimplemented!();
            } else {
                unsafe {
                    match (pk_type, val_type) {
                        #slice_from_variants
                    }
                }
            }
        }
    };

    let init_slice_fn = quote! {
        pub fn init_column_slice<'a,'b: 'a>(
            pk_type: DataType,
            val_type: DataType,
            is_secondary_key: bool,
            slice: &'b mut [u8],
        ) -> Result<Box<dyn ColumnTrait + 'a>, Error> {
            if is_secondary_key {
                unimplemented!();
            } else {
                match (pk_type, val_type) {
                    #slice_init_variants
                }
            }
        }
    };

    quote!(
        #attrs
        #enumeration

        #impl_enum

        #holder_enum

        #column_trait_implementations

        #init_slice_fn

        #from_slice_fn
    )
}

fn get_params(var: &mut Variant) -> Option<Params> {
    let attrs = &mut var.attrs;
    let ident = var.ident.clone();

    // Actually, this is a performance nightmare: too mutch unneeded allocations.
    // Good news: this code will be runned only during compilation, so no runtime costs.
    let params_attr = attrs
        .iter()
        .find(|attr| {
            attr.path.get_ident().map(|ident| format!("{}", ident))
                == Some("type_params".to_string())
        })
        .cloned();

    let attr = params_attr?;

    attrs.retain(|attr| {
        attr.path.get_ident().map(|ident| format!("{}", ident)) != Some("type_params".to_string())
    });

    let group = attr.tokens.into_iter().next()?;

    let inner_stream = match group {
        TokenTree::Group(group) => group.stream(),
        ttree => panic!("Unexpected tokens {:?}", ttree),
    };

    let mut tokens = inner_stream.into_iter();

    let typ_tree = tokens.next()?;

    let typ = match typ_tree {
        TokenTree::Ident(ident) => ident,
        ttree => panic!("Unexpected tokens {:?}", ttree),
    };

    tokens.next();

    let size_tree = tokens.next()?;

    let size = match size_tree {
        TokenTree::Literal(literal) => literal,
        ttree => panic!("Unexpected tokens {:?}", ttree),
    };

    Some(Params { ident, typ, size })
}

struct Params {
    ident: Ident,
    typ: Ident,
    size: Literal,
}
