use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::{anychar, char, crlf, space1};
use nom::combinator::{eof, opt};
use nom::sequence::{delimited, pair, separated_pair, terminated};
use nom::{
    bytes::complete::{tag, take_until},
    combinator::value,
    sequence::tuple,
    IResult,
};

// From nom recipes, one line comment // ...
fn eol_comment<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    value(
        (), // Output is thrown away.
        pair(tag("//"), is_not("\n")),
    )
}

fn end_of_line<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, (Option<&'a str>, Option<()>)> {
    terminated(
        tuple((opt(space1), opt(eol_comment()))),
        alt((eof, tag("\n"), crlf)),
    )
}

fn part<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    delimited(char('"'), take_until("\""), char('"'))
}

fn quadruplet<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, (&'a str, &'a str, &'a str, &'a str)>
{
    tuple((
        terminated(part(), space1),
        terminated(part(), space1),
        terminated(part(), space1),
        terminated(part(), end_of_line()),
    ))
}

fn folding<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, (char, char)> {
    terminated(separated_pair(anychar, char('='), anychar), end_of_line())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn recognize_quadruplet_simple() -> Result<(), Box<dyn Error>> {
        let (remains, (part1, part2, part3, part4)) =
            quadruplet()("\"part1\"  \"part2\"\t \"part3\" \"part4\"")?;

        assert_eq!(remains, "");
        assert_eq!(part1, "part1");
        assert_eq!(part2, "part2");
        assert_eq!(part3, "part3");
        assert_eq!(part4, "part4");

        Ok(())
    }

    #[test]
    fn recognize_quadruplet_with_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, (part1, part2, part3, part4)) =
            quadruplet()("\"part1\"  \"part2\"\t \"part3\" \"part4|part5\"\nOther data")?;

        assert_eq!(remains, "Other data");
        assert_eq!(part1, "part1");
        assert_eq!(part2, "part2");
        assert_eq!(part3, "part3");
        assert_eq!(part4, "part4|part5");

        Ok(())
    }

    #[test]
    fn recognize_quadruplet_with_comment() -> Result<(), Box<dyn Error>> {
        let (remains, (part1, part2, part3, part4)) =
            quadruplet()("\"part1\"  \"part2\"\t \"part3\" \"part4\" \t// This is a comment")?;

        assert_eq!(remains, "");
        assert_eq!(part1, "part1");
        assert_eq!(part2, "part2");
        assert_eq!(part3, "part3");
        assert_eq!(part4, "part4");

        Ok(())
    }

    #[test]
    fn recognize_quadruplet_missing_part() {
        let result: IResult<&str, (&str, &str, &str, &str)> =
            quadruplet()("\"part1\"  \"part2\"\t \"part3\" \t// This is a comment\nOther data");

        assert!(result.is_err());
    }

    #[test]
    fn recognize_quadruplet_failing() {
        let result: IResult<&str, (&str, &str, &str, &str)> =
            quadruplet()("// This is a comment \"part1\"  \"part2\"\t \"part3\"");

        assert!(result.is_err());
    }

    #[test]
    fn folding_simple() -> Result<(), Box<dyn Error>> {
        let (remains, (ch1, ch2)) = folding()("ß=s")?;

        assert_eq!(remains, "");
        assert_eq!(ch1, 'ß');
        assert_eq!(ch2, 's');

        Ok(())
    }

    #[test]
    fn folding_with_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, (ch1, ch2)) = folding()("ó=o\nOther data")?;

        assert_eq!(remains, "Other data");
        assert_eq!(ch1, 'ó');
        assert_eq!(ch2, 'o');

        Ok(())
    }

    #[test]
    fn folding_with_comments() -> Result<(), Box<dyn Error>> {
        let (remains, (ch1, ch2)) = folding()("ó=o // This is one line comment")?;

        assert_eq!(remains, "");
        assert_eq!(ch1, 'ó');
        assert_eq!(ch2, 'o');

        Ok(())
    }

    #[test]
    fn folding_missing_char() {
        let result = folding()("ó=");

        assert!(result.is_err())
    }

    #[test]
    fn folding_not_folding() {
        let result = folding()("Blablabla");

        assert!(result.is_err())
    }

    #[test]
    fn empty_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line()("")?;

        assert_eq!(remains, "");

        Ok(())
    }

    #[test]
    fn empty_line_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line()("\nOther data")?;

        assert_eq!(remains, "Other data");

        Ok(())
    }

    #[test]
    fn commented_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line()("   // This is a comment")?;

        assert_eq!(remains, "");

        Ok(())
    }

    #[test]
    fn commented_line_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line()("   // This is a comment\nOther data")?;

        assert_eq!(remains, "Other data");

        Ok(())
    }
}
