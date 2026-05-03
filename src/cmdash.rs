use nom::{
    character::{complete, streaming::tab},
    combinator::map,
    sequence::tuple,
    IResult,
};
pub enum Command {
    Led(PinState),
    Pin { pin: u8, state: PinState },
}

pub enum PinState {
    On,
    Off,
}

fn parse_pin_state(input: &str) -> IResult<&str, PinState> {
    match input {
        "on" => Ok(("".as_ref(), PinState::On)),
        "off" => Ok(("".as_ref(), PinState::Off)),
        _ => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

#[allow(deprecated)]
fn parse(cmd: &str) -> IResult<&str, Command> {
    let (rest, (_, _, state)) = tuple((
        nom::bytes::complete::tag("set"),
        nom::bytes::complete::tag("led"),
        nom::sequence::preceded(complete::space1, parse_pin_state),
    ))(cmd)?;
    Ok((rest, Command::Led(state)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_led_on() {
        assert_eq!(parse("set led on"), Ok(("", Command::Led(PinState::On))));
    }

    #[test]
    fn test_parse_led_off() {
        assert_eq!(parse("set led off"), Ok(("", Command::Led(PinState::Off))));
    }
}
