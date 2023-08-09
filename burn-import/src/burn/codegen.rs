use proc_macro2::TokenStream;
use quote::quote;

use burn::nn::PaddingConfig2d;

fn convert_primitive<T: ToString>(primitive: T) -> TokenStream {
    let value = primitive.to_string();
    value.parse().unwrap()
}

fn convert_to_array<'a, I, T: ToTokens>(list: I) -> TokenStream
where
    I: Iterator<Item = &'a T>,
    T: 'a,
{
    let mut body = quote! {};

    list.for_each(|item| {
        let elem = item.to_tokens();
        body.extend(quote! {#elem,});
    });

    quote! {
        [#body]
    }
}

pub trait ToTokens {
    fn to_tokens(&self) -> TokenStream;
}

impl<const N: usize, T: Copy + ToTokens> ToTokens for [T; N] {
    fn to_tokens(&self) -> TokenStream {
        convert_to_array(self.iter())
    }
}

impl<T: Copy + ToTokens> ToTokens for Vec<T> {
    fn to_tokens(&self) -> TokenStream {
        convert_to_array(self.iter())
    }
}

/// Prettier output for `usize`
impl ToTokens for usize {
    fn to_tokens(&self) -> TokenStream {
        convert_primitive(self)
    }
}

/// Prettier output for `i64`
impl ToTokens for i64 {
    fn to_tokens(&self) -> TokenStream {
        convert_primitive(self)
    }
}

/// Prettier output for `f64`
impl ToTokens for f64 {
    fn to_tokens(&self) -> TokenStream {
        convert_primitive(self)
    }
}

/// Padding configuration
impl ToTokens for PaddingConfig2d {
    fn to_tokens(&self) -> TokenStream {
        match self {
            Self::Same => quote! { PaddingConfig2d::Same },
            Self::Valid => quote! { PaddingConfig2d::Valid },
            Self::Explicit(padding1, padding2) => {
                let padding1 = padding1.to_tokens();
                let padding2 = padding2.to_tokens();
                quote! { PaddingConfig2d::Explicit(#padding1, #padding2) }
            }
        }
    }
}
