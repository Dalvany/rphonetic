use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_till1, take_until, take_while1};
use nom::character::complete::{alpha1, anychar, char, crlf, space1};
use nom::combinator::{eof, map, map_opt, opt, value};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};

/// From `nom` recipe.
/// Recognize a multiline comment and return the number of lines.
///
/// Multiline comments :
/// ```norust
/// /*
/// ...
/// ...
/// */
/// ```
pub fn multiline_comment<'a>(
) -> impl nom::Parser<&'a str, Output = usize, Error = nom::error::Error<&'a str>> {
    terminated(
        map(
            (tag("/*"), take_until("*/"), tag("*/")),
            |(_, comment, _): (_, &str, _)| comment.split('\n').count(),
        ),
        end_of_line(),
    )
}

/// From `nom` recipe.
/// Recognize a single line comment and discard it
///
/// Single line comment :
/// ```norust
/// // ...
/// ```
fn eol_comment<'a>() -> impl nom::Parser<&'a str, Output = (), Error = nom::error::Error<&'a str>> {
    value(
        (), // Output is thrown away.
        pair(tag("//"), opt(is_not("\n"))),
    )
}

/// Recognize a string that is `true` and return the boolean value.
fn boolean_true<'a>() -> impl nom::Parser<&'a str, Output = bool, Error = nom::error::Error<&'a str>>
{
    map_opt(tag("true"), |_: &str| Some(true))
}

/// Recognize a string that is `false` and return the boolean value.
fn boolean_false<'a>(
) -> impl nom::Parser<&'a str, Output = bool, Error = nom::error::Error<&'a str>> {
    map_opt(tag("false"), |_: &str| Some(false))
}

/// Recognize the end of line.
/// This might be a single line comment or spaces,
/// followed by a `\n`, end of file or `\r\n`.
///
/// When used at the start of a line, if it matches, the line could be considered as empty.
pub fn end_of_line<'a>(
) -> impl nom::Parser<&'a str, Output = (Option<&'a str>, Option<()>), Error = nom::error::Error<&'a str>>
{
    terminated(
        (opt(space1), opt(eol_comment())),
        alt((eof, tag("\n"), crlf)),
    )
}

/// Recognize something between two double quote (`"..."`).
fn part<'a>() -> impl nom::Parser<&'a str, Output = &'a str, Error = nom::error::Error<&'a str>> {
    // There is only "\"" in rules, so to keep thing simple, we will just alt between
    // tag("\\\"") and take_until("\"").
    delimited(char('"'), alt((tag("\\\""), take_until("\""))), char('"'))
}

/// Recognize a quadruplet rule (`"..." "..." "..." "..."`). It could be followed by a single line comment.
///
/// This is a valide Daitch-Mokotoff or Beider-Morse rule.
pub fn quadruplet<'a>() -> impl nom::Parser<
    &'a str,
    Output = (&'a str, &'a str, &'a str, &'a str),
    Error = nom::error::Error<&'a str>,
> {
    (
        terminated(part(), space1),
        terminated(part(), space1),
        terminated(part(), space1),
        terminated(part(), end_of_line()),
    )
}

/// Recognize a Daitch-Mokotoff folding rule (`a=b`). It could be followed by a single line comment.
pub fn folding<'a>(
) -> impl nom::Parser<&'a str, Output = (char, char), Error = nom::error::Error<&'a str>> {
    terminated(separated_pair(anychar, char('='), anychar), end_of_line())
}

/// Recognize a Beider-Morse language detection rule. It could be followed by a single line comment.
pub fn lang<'a>(
) -> impl nom::Parser<&'a str, Output = (&'a str, &'a str, bool), Error = nom::error::Error<&'a str>>
{
    (
        terminated(take_till1(|ch| ch == ' '), char(' ')),
        terminated(take_till1(|ch| ch == ' '), char(' ')),
        terminated(alt((boolean_true(), boolean_false())), end_of_line()),
    )
}

/// Recognize #include for Beider-Morse
pub fn include<'a>(
) -> impl nom::Parser<&'a str, Output = &'a str, Error = nom::error::Error<&'a str>> {
    terminated(
        preceded(
            tag("#include "),
            take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
        ),
        end_of_line(),
    )
}

/// Recognize language for Beider-Morse
pub fn language<'a>(
) -> impl nom::Parser<&'a str, Output = &'a str, Error = nom::error::Error<&'a str>> {
    terminated(alpha1, end_of_line())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use nom::{IResult, Parser};

    use super::*;

    #[test]
    fn test_quadruplet_simple() -> Result<(), Box<dyn Error>> {
        let (remains, (part1, part2, part3, part4)) =
            quadruplet().parse("\"part1\"  \"part2\"\t \"part3\" \"part4\"")?;

        assert_eq!(remains, "");
        assert_eq!(part1, "part1");
        assert_eq!(part2, "part2");
        assert_eq!(part3, "part3");
        assert_eq!(part4, "part4");

        Ok(())
    }

    #[test]
    fn test_quadruplet_with_backslash_double_quote() -> Result<(), Box<dyn Error>> {
        let (remains, (part1, part2, part3, part4)) =
            quadruplet().parse("\"\\\"\"  \"\" \"\" \"\"")?;

        assert_eq!(remains, "");
        assert_eq!(part1, "\\\"");
        assert_eq!(part2, "");
        assert_eq!(part3, "");
        assert_eq!(part4, "");

        Ok(())
    }

    #[test]
    fn test_quadruplet_with_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, (part1, part2, part3, part4)) =
            quadruplet().parse("\"part1\"  \"part2\"\t \"part3\" \"part4|part5\"\nOther data")?;

        assert_eq!(remains, "Other data");
        assert_eq!(part1, "part1");
        assert_eq!(part2, "part2");
        assert_eq!(part3, "part3");
        assert_eq!(part4, "part4|part5");

        Ok(())
    }

    #[test]
    fn test_quadruplet_with_comment() -> Result<(), Box<dyn Error>> {
        let (remains, (part1, part2, part3, part4)) = quadruplet()
            .parse("\"part1\"  \"part2\"\t \"part3\" \"part4\" \t// This is a comment")?;

        assert_eq!(remains, "");
        assert_eq!(part1, "part1");
        assert_eq!(part2, "part2");
        assert_eq!(part3, "part3");
        assert_eq!(part4, "part4");

        Ok(())
    }

    #[test]
    fn test_quadruplet_missing_part() {
        let result: IResult<&str, (&str, &str, &str, &str)> = quadruplet()
            .parse("\"part1\"  \"part2\"\t \"part3\" \t// This is a comment\nOther data");

        assert!(result.is_err());
    }

    #[test]
    fn test_quadruplet_failing() {
        let result: IResult<&str, (&str, &str, &str, &str)> =
            quadruplet().parse("// This is a comment \"part1\"  \"part2\"\t \"part3\"");

        assert!(result.is_err());
    }

    #[test]
    fn test_quadruplet_inside_comment_should_fail() {
        let result = quadruplet().parse("//\"part1\"  \"part2\"\t \"part3\" \"part4\"");

        assert!(result.is_err());
    }

    #[test]
    fn test_folding_simple() -> Result<(), Box<dyn Error>> {
        let (remains, (ch1, ch2)) = folding().parse("ß=s")?;

        assert_eq!(remains, "");
        assert_eq!(ch1, 'ß');
        assert_eq!(ch2, 's');

        Ok(())
    }

    #[test]
    fn test_folding_with_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, (ch1, ch2)) = folding().parse("ó=o\nOther data")?;

        assert_eq!(remains, "Other data");
        assert_eq!(ch1, 'ó');
        assert_eq!(ch2, 'o');

        Ok(())
    }

    #[test]
    fn test_folding_with_comments() -> Result<(), Box<dyn Error>> {
        let (remains, (ch1, ch2)) = folding().parse("ó=o // This is one line comment")?;

        assert_eq!(remains, "");
        assert_eq!(ch1, 'ó');
        assert_eq!(ch2, 'o');

        Ok(())
    }

    #[test]
    fn test_folding_missing_char() {
        let result = folding().parse("ó=");

        assert!(result.is_err())
    }

    #[test]
    fn test_folding_not_folding() {
        let result = folding().parse("Blablabla");

        assert!(result.is_err())
    }

    #[test]
    fn test_folding_inside_comment_should_fail() {
        let result = folding().parse("//a=b");

        assert!(result.is_err());
    }

    #[test]
    fn test_empty_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line().parse("")?;

        assert_eq!(remains, "");

        Ok(())
    }

    #[test]
    fn test_empty_line_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line().parse("\nOther data")?;

        assert_eq!(remains, "Other data");

        Ok(())
    }

    #[test]
    fn test_commented_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line().parse("   // This is a comment")?;

        assert_eq!(remains, "");

        Ok(())
    }

    #[test]
    fn test_commented_line_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line().parse("   //This is a comment\nOther data")?;

        assert_eq!(remains, "Other data");

        Ok(())
    }

    #[test]
    fn test_empty_comment_with_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line().parse("//\nOther data")?;

        assert_eq!(remains, "Other data");

        Ok(())
    }

    #[test]
    fn test_empty_comment() -> Result<(), Box<dyn Error>> {
        let (remains, _) = end_of_line().parse("//")?;

        assert_eq!(remains, "");

        Ok(())
    }

    #[test]
    fn test_lang_simple_true() -> Result<(), Box<dyn Error>> {
        let (remains, (condition, languages, include)) =
            lang().parse("zh polish+russian+german+english true")?;

        assert_eq!(remains, "");
        assert_eq!(condition, "zh");
        assert_eq!(languages, "polish+russian+german+english");
        assert!(include);

        Ok(())
    }

    #[test]
    fn test_lang_simple_false() -> Result<(), Box<dyn Error>> {
        let (remains, (condition, languages, include)) =
            lang().parse("zh polish+russian+german+english false")?;

        assert_eq!(remains, "");
        assert_eq!(condition, "zh");
        assert_eq!(languages, "polish+russian+german+english");
        assert!(!include);

        Ok(())
    }

    #[test]
    fn test_lang_with_other_line() -> Result<(), Box<dyn Error>> {
        let (remains, (condition, languages, include)) =
            lang().parse("zh polish+russian+german+english true\nOther data")?;

        assert_eq!(remains, "Other data");
        assert_eq!(condition, "zh");
        assert_eq!(languages, "polish+russian+german+english");
        assert!(include);

        Ok(())
    }

    #[test]
    fn test_lang_with_comment() -> Result<(), Box<dyn Error>> {
        let (remains, (condition, languages, include)) = lang()
            .parse("zh polish+russian+german+english true // This is a comment\nOther data")?;

        assert_eq!(remains, "Other data");
        assert_eq!(condition, "zh");
        assert_eq!(languages, "polish+russian+german+english");
        assert!(include);

        Ok(())
    }

    #[test]
    fn test_lang_missing_part() {
        let result = lang().parse("zh true // This is a comment\nOther data");

        assert!(result.is_err())
    }

    #[test]
    fn test_lang_not_lang() {
        let result = lang().parse("// This is a comment");

        assert!(result.is_err())
    }

    // Will be checked before feeding the line.
    #[ignore]
    #[test]
    fn test_lang_inside_comment_should_fail() {
        let result = lang().parse("//zh polish+russian+german+english true");

        assert!(result.is_err())
    }

    #[test]
    fn test_multiline_comment() -> Result<(), Box<dyn Error>> {
        let (remains, line_count) = multiline_comment().parse(
            "/* This\n\
        is\n\
        a\n\
        multiline\n\
        comment */",
        )?;

        assert_eq!(remains, "");
        assert_eq!(line_count, 5);

        Ok(())
    }

    #[test]
    fn test_multiline_comment_followed_by_single_line_comment() -> Result<(), Box<dyn Error>> {
        let (remains, line_count) = multiline_comment().parse(
            "/* This\n\
        is\n\
        a\n\
        multiline\n\
        comment */ // This is a single line comment",
        )?;

        assert_eq!(remains, "");
        assert_eq!(line_count, 5);

        Ok(())
    }
}
