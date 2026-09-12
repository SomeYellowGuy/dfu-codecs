use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    let mut stream = TokenStream::new();

    for n in 3..=16 {
        let ident = format_ident!("apply_{n}");
        let n_literal = Literal::string(&n.to_string());

        let types = (0..n).into_iter().map(|i| {
            let upper = format_ident!("{}", (b'A' + i) as char);
            let lower = format_ident!("{}", (b'a' + i) as char);
            quote! { #upper #lower }
        });

        let separated_results = (0..n)
            .into_iter()
            .rev()
            .map(|i| format_ident!("{}", (b'a' + i) as char));

        let too_many_arguments = (n >= 7).then_some(quote! { | too_many_arguments });

        stream.extend(quote! {
            apply_data_results!(pub #ident | #n_literal | #(#types),* | #(#separated_results),* #too_many_arguments);
        });
    }

    quote!(
        impl<R> DataResult<R> {
            #stream
        }
    )
}
