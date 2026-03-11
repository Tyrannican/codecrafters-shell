use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{take, take_while, take_while1},
    character::complete::char,
    multi::many0,
    sequence::{delimited, preceded},
};

fn backslash_escape(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    preceded(char('\\'), take(1usize))
        .map(|item: &[u8]| item.to_vec())
        .parse(input)
}

fn single_quote(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    delimited(char('\''), take_while(|c: u8| c != b'\''), char('\''))
        .map(|item: &[u8]| item.to_vec())
        .parse(input)
}

fn escaped_special(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    preceded(
        char('\\'),
        alt((char('\\'), char('$'), char('`'), char('\"'), char('\n'))),
    )
    .map(|item: char| vec![item as u8])
    .parse(input)
}

fn non_escaped(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    take_while1(|c: u8| c != b'"' && c != b'\\')
        .map(|item: &[u8]| item.to_vec())
        .parse(input)
}

fn escaped_non_special(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    preceded(char('\\'), take(1usize))
        .map(|item: &[u8]| item.to_vec())
        .parse(input)
}

fn double_quote(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    delimited(
        char('"'),
        many0(alt((escaped_special, escaped_non_special, non_escaped))),
        char('"'),
    )
    .map(|item: Vec<Vec<u8>>| item.concat())
    .parse(input)
}

fn plain(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    take_while1(|c: u8| !c.is_ascii_whitespace() && c != b'\'' && c != b'"' && c != b'\\')
        .map(|item: &[u8]| item.to_vec())
        .parse(input)
}

fn fragments(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((backslash_escape, single_quote, double_quote, plain)).parse(input)
}

fn word(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    many0(fragments)
        .map(|frag: Vec<Vec<u8>>| frag.concat())
        .parse(input)
}

pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<String>> {
    let mut phrases = Vec::new();
    let mut remaining = input;

    loop {
        let (rest, _) = take_while(|c: u8| c.is_ascii_whitespace()).parse(remaining)?;
        remaining = rest;
        if remaining.is_empty() {
            break;
        }

        let (rest, phrase) = word(remaining)?;
        if phrase.is_empty() {
            break;
        }

        phrases.push(String::from_utf8(phrase).unwrap());
        remaining = rest;
    }

    Ok((remaining, phrases))
}
