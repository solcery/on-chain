use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::quote;
use syn::{Ident, ItemEnum, Variant};

#[proc_macro_attribute]
pub fn generate_column_impls(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let output = column_impls(&TokenStream::from(attrs), input);
    proc_macro::TokenStream::from(output)
}

fn column_impls(attrs: &TokenStream, input: proc_macro::TokenStream) -> TokenStream {
    let (holder_ident, trait_ident, error_ident, derive_attrs) = parse_attrs(attrs);
    let mut enumeration: ItemEnum = syn::parse(input).expect("Failed to parse input");
    let enum_ident = enumeration.ident.clone();
    let variants: Vec<Params> = enumeration
        .variants
        .iter_mut()
        .map(|var| parse_params(var).expect("No parameter attributes present"))
        .collect();

    let holder_enum = generate_holder_enum(&holder_ident, &variants);

    let impl_enum = generate_enum_impl(&enum_ident, &variants);

    let holder_attrs = quote! {
        #[derive #derive_attrs]
    };

    let column_trait_implementations = variants.iter().map(|key| {
        let key_ident = &key.ident;
        let key_type = &key.typ;
        let key_size = &key.size;

        variants.iter().map(|value| {
            let value_ident = &value.ident;
            let value_type = &value.typ;
            let value_size = &value.size;
            quote!{
                impl<'a> #trait_ident for slice_rbtree::tree::RBTree<'a, #key_type, #value_type, #key_size, #value_size> {
                    fn get_key(&self, value: #holder_ident) -> Option<#holder_ident> {
                        //TODO: implement a slow but working variant
                        None
                    }

                    fn get_value(&self, key: #holder_ident) -> Option<#holder_ident> {
                        if let #holder_ident::#key_ident(unwrapped_key) = key {
                            self.get(&unwrapped_key).map(|val| #holder_ident::#value_ident(val))
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn set(&mut self, key: #holder_ident, value: #holder_ident) -> Result<Option<#holder_ident>, #error_ident> {
                        if let (#holder_ident::#key_ident(unwrapped_key), #holder_ident::#value_ident(unwrapped_value)) = (key, value) {
                            self.insert(unwrapped_key, unwrapped_value)
                                .map(|maybe_old_val| maybe_old_val.map(|old_val|#holder_ident::#value_ident(old_val)))
                                .map_err(|e| #error_ident::from(e))
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn delete_by_key(&mut self, key: #holder_ident) -> bool {
                        if let #holder_ident::#key_ident(unwrapped_key) = key {
                            self.delete(&unwrapped_key)
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn delete_by_value(&mut self, value: #holder_ident) -> bool {
                        //TODO: implement a slow but working variant
                        panic!("It is impossible to delete by value in RBTree");
                    }
                }
                impl<'a> #trait_ident for solcery_reltab::one_to_one::OneToOne<'a, #key_type, #value_type, #key_size, #value_size> {
                    fn get_key(&self, value: #holder_ident) -> Option<#holder_ident> {
                        if let #holder_ident::#value_ident(unwrapped_value) = value {
                            self.get_key(&unwrapped_value).map(|key| #holder_ident::#key_ident(key))
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn get_value(&self, key: #holder_ident) -> Option<#holder_ident> {
                        if let #holder_ident::#key_ident(unwrapped_key) = key {
                            self.get_value(&unwrapped_key).map(|val| #holder_ident::#value_ident(val))
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn set(&mut self, key: #holder_ident, value: #holder_ident) -> Result<Option<#holder_ident>, #error_ident> {
                        if let (#holder_ident::#key_ident(unwrapped_key), #holder_ident::#value_ident(unwrapped_value)) = (key, value) {
                            //TODO: Do something with tis allocations
                            let result = self.insert(unwrapped_key.clone(), unwrapped_value.clone());
                            match result {
                                Ok((Some(old_key), None)) => {
                                    self.insert(old_key, unwrapped_value).expect("Failed to revert changes, DB is now corrupted");
                                    Err(#error_ident::NonUniqueSecondaryKey)
                                }
                                Ok((Some(old_key), Some(old_value))) => {
                                    self.insert(old_key, unwrapped_value).expect("Failed to revert changes, DB is now corrupted");
                                    self.insert(unwrapped_key, old_value).expect("Failed to revert changes, DB is now corrupted");
                                    Err(#error_ident::NonUniqueSecondaryKey)
                                }
                                Ok((None, maybe_old_val)) => {
                                    Ok(maybe_old_val.map(|old_val|#holder_ident::#value_ident(old_val)))
                                }
                                Err(e) => {
                                    Err(#error_ident::from(e))
                                }
                            }
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn delete_by_key(&mut self, key: #holder_ident) -> bool {
                        if let #holder_ident::#key_ident(unwrapped_key) = key {
                            self.remove_by_key(&unwrapped_key).is_some()
                        } else {
                            panic!("Type mismatch!");
                        }
                    }

                    fn delete_by_value(&mut self, value: #holder_ident) -> bool {
                        if let #holder_ident::#value_ident(unwrapped_value) = value {
                            self.remove_by_value(&unwrapped_value).is_some()
                        } else {
                            panic!("Type mismatch!");
                        }
                    }
                }
            }
        }).collect::<TokenStream>()
    }).collect::<TokenStream>();

    let result: ((TokenStream, TokenStream), (TokenStream, TokenStream))  = variants.iter().map(|key| {
        let key_ident = key.ident.clone();
        let key_type = key.typ.clone();
        let key_size = key.size.clone();

        variants.iter().map(|value| {
            let value_ident = value.ident.clone();
            let value_type = value.typ.clone();
            let value_size = value.size.clone();

            let slice_init_variant = quote! {
                (#enum_ident::#key_ident, #enum_ident::#value_ident) => slice_rbtree::tree::RBTree::<#key_type, #value_type, #key_size, #value_size>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn #trait_ident>)
                    .map_err(|e| #error_ident::from(e)),
            };

            let slice_from_variant = quote! {
                (#enum_ident::#key_ident, #enum_ident::#value_ident) => slice_rbtree::tree::RBTree::<#key_type, #value_type, #key_size, #value_size>::from_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn #trait_ident>)
                    .map_err(|e| #error_ident::from(e)),
            };

            let one_to_one_init_variant = quote! {
                (#enum_ident::#key_ident, #enum_ident::#value_ident) => solcery_reltab::one_to_one::OneToOne::<#key_type, #value_type, #key_size, #value_size>::init_slice(slice)
                    .map(|container| Box::new(container) as Box<dyn #trait_ident>)
                    .map_err(|e| #error_ident::from(e)),
            };

            let one_to_one_from_variant = quote! {
                (#enum_ident::#key_ident, #enum_ident::#value_ident) => solcery_reltab::one_to_one::OneToOne::<#key_type, #value_type, #key_size, #value_size>::from_slice(slice)
                    .map(|container| Box::new(container) as Box<dyn #trait_ident>)
                    .map_err(|e| #error_ident::from(e)),
            };

            (
                (slice_init_variant, slice_from_variant),
                (one_to_one_init_variant, one_to_one_from_variant)
            )
        })
        .unzip::<(TokenStream, TokenStream), (TokenStream, TokenStream), (TokenStream, TokenStream), (TokenStream, TokenStream)>()
    }).unzip();

    let (
        (slice_init_variants, slice_from_variants),
        (one_to_one_init_variants, one_to_one_from_variants),
    ) = result;

    let from_slice_fn = quote! {
        pub fn from_column_slice<'a,'b: 'a>(
            pk_type: #enum_ident,
            val_type: #enum_ident,
            column_type: ColumnType,
            slice: &'b mut [u8],
        ) -> Result<Box<dyn #trait_ident + 'a>, #error_ident> {
            match column_type {
                ColumnType::RBTree => {
                    unsafe {
                        match (pk_type, val_type) {
                            #slice_from_variants
                        }
                    }
                },
                ColumnType::OneToOne => {
                    unsafe {
                        match (pk_type, val_type) {
                            #one_to_one_from_variants
                        }
                    }
                },
            }
        }
    };

    let init_slice_fn = quote! {
        pub fn init_column_slice<'a,'b: 'a>(
            pk_type: #enum_ident,
            val_type: #enum_ident,
            column_type: ColumnType,
            slice: &'b mut [u8],
        ) -> Result<Box<dyn #trait_ident + 'a>, #error_ident> {
            match column_type {
                ColumnType::RBTree => {
                    match (pk_type, val_type) {
                        #slice_init_variants
                    }
                },
                ColumnType::OneToOne => {
                    unsafe {
                        match (pk_type, val_type) {
                            #one_to_one_init_variants
                        }
                    }
                },
            }
        }
    };

    quote!(
        #enumeration

        #impl_enum

        #holder_attrs
        #holder_enum

        #column_trait_implementations

        #init_slice_fn

        #from_slice_fn
    )
}

fn parse_params(var: &mut Variant) -> Option<Params> {
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

#[derive(Debug, Clone)]
struct Params {
    ident: Ident,
    typ: Ident,
    size: Literal,
}

fn generate_holder_enum(holder_ident: &Ident, variants: &[Params]) -> TokenStream {
    let holder_vars: TokenStream = variants
        .iter()
        .map(|var| {
            let ident = &var.ident;
            let typ = &var.typ;

            quote! {
                #ident(#typ),
            }
        })
        .collect();

    quote! {
        pub enum #holder_ident {
            #holder_vars
        }
    }
}

fn generate_enum_impl(enum_ident: &Ident, variants: &[Params]) -> TokenStream {
    let fn_vars: TokenStream = variants
        .iter()
        .map(|var| {
            let ident = &var.ident;
            let size = &var.size;

            quote! {
                #enum_ident::#ident => #size,
            }
        })
        .collect();

    quote! {
        impl #enum_ident {
            pub const fn size(&self) -> usize {
                match self {
                    #fn_vars
                }
            }
        }
    }
}

fn parse_attrs(attrs: &TokenStream) -> (Ident, Ident, Ident, TokenTree) {
    let mut token_iterator = attrs.clone().into_iter();

    let holder_ident = match token_iterator.next() {
        Some(TokenTree::Ident(ident)) => ident,
        Some(tokens) => panic!("Unexpected tokens: {}", tokens),
        None => panic!("Not enough arguments"),
    };

    token_iterator.next(); // skip punct

    let trait_ident = match token_iterator.next() {
        Some(TokenTree::Ident(ident)) => ident,
        Some(tokens) => panic!("Unexpected tokens: {}", tokens),
        None => panic!("Not enough arguments"),
    };

    token_iterator.next(); // skip punct

    let error_ident = match token_iterator.next() {
        Some(TokenTree::Ident(ident)) => ident,
        Some(tokens) => panic!("Unexpected tokens: {}", tokens),
        None => panic!("Not enough arguments"),
    };

    token_iterator.next(); // skip punct

    match token_iterator.next() {
        Some(TokenTree::Ident(ident)) => {
            if format!("{}", ident) != *"derives" {
                panic!("Unexpected keyword: {}", ident)
            }
        }
        Some(tokens) => panic!("Unexpected tokens: {}", tokens),
        None => panic!("Not enough arguments"),
    }

    let derives = match token_iterator.next() {
        Some(TokenTree::Group(group)) => TokenTree::Group(group),
        Some(tokens) => panic!("Unexpected tokens: {}", tokens),
        None => panic!("Not enough arguments"),
    };

    (holder_ident, trait_ident, error_ident, derives)
}
