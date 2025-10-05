use super::error::Error;
use super::ids::{BaseClass, Device, ProgIf, SubClass, SubSystem, Vendor};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until, take_until1},
    character::complete::hex_digit1,
    combinator::map,
    multi::{many0, many1},
    sequence::preceded,
};

pub fn parse(input: &str) -> Result<(Vec<Vendor>, Vec<BaseClass>), Error> {
    let (rest, vencors) = many1(vendor_block).parse(input)?;
    let (rest, classes) = many1(class_block).parse(rest)?;
    let (rest, _) = many0(tag("\n")).parse(rest)?;
    if !rest.is_empty() {
        return Err(Error::TrailingData);
    }
    Ok((vencors, classes))
}

fn vendor_id(input: &str) -> IResult<&str, u16> {
    map(
        preceded(many0(alt((tag("\n"), comment))), hex_digit1),
        |v: &str| u16::from_str_radix(v, 16).unwrap(),
    )
    .parse(input)
}

fn device_id(input: &str) -> IResult<&str, u16> {
    map(
        preceded((many0(alt((tag("\n"), comment))), tag("\t")), hex_digit1),
        |v: &str| u16::from_str_radix(v, 16).unwrap(),
    )
    .parse(input)
}

fn subvendor_id(input: &str) -> IResult<&str, u16> {
    map(
        preceded((many0(alt((tag("\n"), comment))), tag("\t\t")), hex_digit1),
        |v: &str| u16::from_str_radix(v, 16).unwrap(),
    )
    .parse(input)
}

fn subdevice_id(input: &str) -> IResult<&str, u16> {
    map(hex_digit1, |v: &str| u16::from_str_radix(v, 16).unwrap()).parse(input)
}

fn class_id(input: &str) -> IResult<&str, u8> {
    map(
        preceded((many0(alt((tag("\n"), comment))), tag("C ")), hex_digit1),
        |v: &str| u8::from_str_radix(v, 16).unwrap(),
    )
    .parse(input)
}

fn subclass_id(input: &str) -> IResult<&str, u8> {
    map(
        preceded((many0(alt((tag("\n"), comment))), tag("\t")), hex_digit1),
        |v: &str| u8::from_str_radix(v, 16).unwrap(),
    )
    .parse(input)
}

fn progif_id(input: &str) -> IResult<&str, u8> {
    map(
        preceded((many0(alt((tag("\n"), comment))), tag("\t\t")), hex_digit1),
        |v: &str| u8::from_str_radix(v, 16).unwrap(),
    )
    .parse(input)
}

fn str_to_eol(input: &str) -> IResult<&str, &str> {
    take_until1("\n").parse(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    map((tag("#"), take_until("\n")), |(_, _)| "").parse(input)
}

fn vendor_block(input: &str) -> IResult<&str, Vendor> {
    map(
        (
            vendor_id,
            preceded(tag("  "), str_to_eol),
            many0(device_block),
        ),
        |(i, n, d)| Vendor::new(i, n.to_string(), d),
    )
    .parse(input)
}

fn device_block(input: &str) -> IResult<&str, Device> {
    map(
        (
            device_id,
            preceded(tag("  "), str_to_eol),
            many0(subsystem_block),
        ),
        |(i, n, s)| Device::new(i, n.to_string(), s),
    )
    .parse(input)
}

fn subsystem_block(input: &str) -> IResult<&str, SubSystem> {
    map(
        (
            subvendor_id,
            preceded(tag(" "), subdevice_id),
            preceded(tag("  "), str_to_eol),
        ),
        |(v, d, n)| SubSystem::new(v, d, n.to_string()),
    )
    .parse(input)
}

fn class_block(input: &str) -> IResult<&str, BaseClass> {
    map(
        (
            class_id,
            preceded(tag("  "), str_to_eol),
            many0(subclass_block),
        ),
        |(i, n, s)| BaseClass::new(i, n.to_string(), s),
    )
    .parse(input)
}

fn subclass_block(input: &str) -> IResult<&str, SubClass> {
    map(
        (
            subclass_id,
            preceded(tag("  "), str_to_eol),
            many0(progif_block),
        ),
        |(i, n, p)| SubClass::new(i, n.to_string(), p),
    )
    .parse(input)
}

fn progif_block(input: &str) -> IResult<&str, ProgIf> {
    map((progif_id, preceded(tag("  "), str_to_eol)), |(i, n)| {
        ProgIf::new(i, n.to_string())
    })
    .parse(input)
}
