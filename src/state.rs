use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Result, Token,
};

#[derive(Debug)]
pub struct State {
    pub name: Ident,
    pub variant: Ident,
    pub extra: Option<TokenStream>,
}

impl Parse for State {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        input.parse::<Token![::]>()?;

        let variant: Ident = input.parse()?;

        let extra = if input.is_empty() {
            None
        } else {
            input.parse::<Token!(,)>()?;
            Some(input.parse::<TokenStream>()?)
        };

        Ok(State {
            name,
            variant,
            extra,
        })
    }
}
