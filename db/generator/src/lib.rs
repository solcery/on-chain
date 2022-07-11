use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, ImplItem, ItemEnum, ItemFn, ItemImpl, Result, Token, Variant};

#[proc_macro_attribute]
pub fn generate_column_impls(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let output = column_impls(&TokenStream::from(attr), input);
    proc_macro::TokenStream::from(output)
}

fn column_impls(attr: &TokenStream, input: proc_macro::TokenStream) -> TokenStream {
    let mut enumeration: ItemEnum = syn::parse(input).expect("Failed to parse input");
    let enum_ident = enumeration.ident.clone();
    let variants: Vec<Params> = enumeration
        .variants
        .iter_mut()
        .map(|var| get_params(var).expect("No parameter attributes present"))
        .collect();

    let (holder_vars, fn_vars): (TokenStream, TokenStream) = variants
        .into_iter()
        .map(|var| {
            let ident = var.ident;
            let typ = var.typ;
            let size = var.size;

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
        enum Data {
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

    //todo!();
    let result = quote!(
        #attrs
        #enumeration
        #holder_enum
        #impl_enum
    );
    println!("{}", result);
    result
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
        .map(|attr| attr.clone());

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
