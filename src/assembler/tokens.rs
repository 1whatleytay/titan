use std::collections::HashMap;
use nom::error::Error;
use nom::{IResult, Parser};
use crate::assembler::labels::label_name;

pub struct TokenCache {
    tokens: HashMap<String, String>
}

impl TokenCache {
    pub fn new() -> TokenCache {
        TokenCache { tokens: HashMap::new() }
    }
}

pub fn token<'a, F, O>(mut f: F, cache: &'a TokenCache)
    -> impl FnMut(&'a str) -> IResult<&'a str, O> where
        F: Parser<&'a str, O, Error<&'a str>>
{
    move |input| {
        match f.parse(input) {
            Ok(result) => Ok(result),
            Err(error) => {
                let result: IResult<&str, &str> = label_name(input);

                if let Ok((_, token_string)) = result {
                    if let Some(result) = cache.tokens.get(token_string) {
                        return f.parse(result)
                    }
                }

                Err(error)
            }
        }
    }
}

pub fn token_lookup<'a, F, O>(mut f: F, cache: &'a TokenCache)
                       -> impl FnMut(&'a str) -> IResult<&'a str, O> where
    F: Parser<&'a str, O, Error<&'a str>>
{
    move |input| {
        let result: IResult<&str, &str> = label_name(input);

        if let Ok((_, token_string)) = result {
            if let Some(result) = cache.tokens.get(token_string) {
                return f.parse(result)
            }
        }

        f.parse(input)
    }
}

pub fn with_cache<'a, F, O>(mut f: F, cache: &'a TokenCache)
    -> impl FnMut(&'a str) -> IResult<&'a str, O> where
        F: FnMut(&'a str, &'a TokenCache) -> IResult<&'a str, O> {
    move |input| {
        f(input, cache)
    }
}
