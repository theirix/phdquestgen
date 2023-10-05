//TODO drop later
#![allow(unused_variables, unused_imports, dead_code)]

use anyhow::anyhow;
use log::info;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{alphanumeric1, line_ending, multispace0, newline, not_line_ending},
    combinator::{map, map_res, opt, value},
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, terminated, tuple},
    IResult, Parser,
};
use std::convert::AsRef;

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Option {
    Event,
    Miniboss,
    Boss,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub options: Vec<Option>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    pub action: String,
    pub options: Options,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Stage(Stage),
    Now,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quest {
    pub lines: Vec<Line>,
}

fn parse_other_option(input: &str) -> IResult<&str, Option> {
    let (remaining, s) = alphanumeric1(input)?;
    Ok((remaining, Option::Other(s.to_string())))
}

fn parse_option(input: &str) -> IResult<&str, Option> {
    alt((
        value(Option::Event, tag("event")),
        value(Option::Boss, tag("boss")),
        value(Option::Miniboss, tag("miniboss")),
        parse_other_option,
    ))(input)
}

fn parse_options(input: &str) -> IResult<&str, Options> {
    let (remaining, options) =
        delimited(tag("["), separated_list0(tag(","), parse_option), tag("]"))(input)?;
    Ok((remaining, Options { options }))
}

fn parse_stage(input: &str) -> IResult<&str, Stage> {
    let (remaining, (_, options, stage)) = tuple((
        multispace0,
        opt(parse_options),
        preceded(multispace0, not_line_ending),
    ))(input)?;
    let options = match options {
        Some(val) => val,
        None => Options { options: vec![] },
    };
    Ok((
        remaining,
        Stage {
            action: stage.to_string(),
            options,
        },
    ))
}

fn parse_line(input: &str) -> IResult<&str, Line> {
    alt((
        value(Line::Now, tuple((multispace0, tag("---")))),
        map(parse_stage, Line::Stage),
    ))(input)
}

fn parse_quest(input: &str) -> IResult<&str, Quest> {
    let (remaining, lines) = separated_list0(newline, parse_line)(input)?;
    Ok((remaining, Quest { lines }))
}

pub fn parse(quest_string: String) -> anyhow::Result<Quest> {
    let (rem, quest) = parse_quest(quest_string.as_str()).unwrap();
    info!("Rem: {}. Data: {:?}", &rem, &quest);
    if !rem.is_empty() {
        Err(anyhow!(format!("Parse failed with remaining {}", rem)))
    } else {
        Ok(quest)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use test_log::test;

    static SAMPLE1: &str = r"first
[event] second
 ---
  third
[event,boss] third";

    #[test]
    fn test_parse_one() {
        let (remaining, line) = parse_line("[event] Foo bar".into()).expect("parsed");
        assert_eq!(remaining.len(), 0);
        match line {
            Line::Stage(stage) => {
                assert_eq!(stage.action, "Foo bar");
                assert_eq!(
                    stage.options,
                    Options {
                        options: vec![Option::Event]
                    }
                );
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn test_parse_no_option() {
        let (remaining, line) = parse_line("Foo bar".into()).expect("parsed");
        assert_eq!(remaining.len(), 0);
        match line {
            Line::Stage(stage) => {
                assert_eq!(stage.action, "Foo bar");
                assert_eq!(stage.options, Options { options: vec![] });
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn test_parse_now() {
        let (remaining, line) = parse_line("---".into()).expect("parsed");
        assert_eq!(remaining.len(), 0);
        if let Line::Now = line {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_quest_one() {
        let (remaining, quest) = parse_quest("[event] Foo bar".into()).expect("parsed");
        assert_eq!(remaining.len(), 0);
        assert_eq!(quest.lines.len(), 1);
    }

    #[test]
    fn test_parse_quest() {
        let (remaining, quest) = parse_quest(SAMPLE1.into()).expect("parsed");
        assert_eq!(remaining.len(), 0);
        println!("Quest {:?}", &quest);
        assert_eq!(quest.lines.len(), 5);
        if let Line::Stage(stage) = quest.lines.iter().nth(0).unwrap() {
            assert_eq!(
                *stage,
                Stage {
                    action: "first".to_string(),
                    options: Options { options: vec![] }
                }
            );
        } else {
            assert!(false);
        }
        if let Line::Now = quest.lines.iter().nth(2).unwrap() {
            assert!(true);
        } else {
            assert!(false);
        }
        if let Line::Stage(stage) = quest.lines.iter().nth(4).unwrap() {
            assert_eq!(
                *stage,
                Stage {
                    action: "third".to_string(),
                    options: Options {
                        options: vec![Option::Event, Option::Boss]
                    }
                }
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_unknown_option() {
        let (remaining, line) = parse_line("[unknown,boss] Foo bar".into()).expect("parsed");
        assert_eq!(remaining.len(), 0);
        match line {
            Line::Stage(stage) => {
                assert_eq!(stage.action, "Foo bar");
                assert_eq!(
                    stage.options,
                    Options {
                        options: vec![Option::Other("unknown".into()), Option::Boss]
                    }
                );
            }
            _ => assert!(false),
        };
    }
}
