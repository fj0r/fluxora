use syn::{
    Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct Pair {
    key: Ident,
    value: LitStr,
}

impl Parse for Pair {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?;
        let _eq_token: Token![=] = input.parse()?;
        let value: LitStr = input.parse()?;
        Ok(Pair { key, value })
    }
}

#[derive(Default)]
pub struct Attrs(pub Vec<(String, String)>);

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let list = Punctuated::<Pair, Token![,]>::parse_terminated(input)?;
        let list = list
            .iter()
            .map(|x| (x.key.to_string(), x.value.value()))
            .collect::<Vec<_>>();
        Ok(Attrs(list))
    }
}
