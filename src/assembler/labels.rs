use nom::bytes::complete::is_a;
use nom::IResult;

pub const LABEL_CHARACTERS: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_";

pub fn label_name(input: &str) -> IResult<&str, &str> {
    is_a(LABEL_CHARACTERS)(input)
}
