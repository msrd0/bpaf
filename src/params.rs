/// Primitives to define parsers
///
/// # Terminology
///
/// ## Flag
///
/// A simple no-argument command line option that takes no extra parameters, when decoded produces
/// a fixed value. Can have a short (`-f`) or a long (`--flag`) name, see [`Named::flag`] and
/// [`Named::req_flag`].
///
/// ## Switch
///
/// A special case of a flag that gets decoded into a `bool`, see [`Named::switch`] and
/// [`Named::req_switch`]
///
/// ## Option
///
/// A command line option with a name that also takes a value. Can have a short (`-f value`) or a
/// long (`--flag value`) name, see [`Named::argument`].
///
/// ## Argument
///
/// A positional argument with no additonal name, for example in `vim main.rs` a command `main.rs`
/// is a positional argument. See [`positional`].
///
/// ## Command
///
/// A command is used to define a starting point for an independent subparser, for example in
/// `cargo check --workspace` `check` defines a subparser that acceprts `--workspace` switch. See
/// [`command`]
///
use std::ffi::OsString;

use super::*;
use crate::{
    args::Word,
    info::{ItemKind, Meta},
};

#[derive(Clone, Debug)]
pub struct Named {
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
}

pub fn short(short: char) -> Named {
    Named {
        short: vec![short],
        long: Vec::new(),
        help: None,
    }
}

pub fn long(long: &'static str) -> Named {
    Named {
        short: Vec::new(),
        long: vec![long],
        help: None,
    }
}

impl Named {
    pub fn short(mut self, short: char) -> Self {
        self.short.push(short);
        self
    }
    pub fn long(mut self, long: &'static str) -> Self {
        self.long.push(long);
        self
    }
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}

impl Named {
    /// simple boolean flag
    pub fn switch(self) -> Flag<bool> {
        Flag {
            present: true,
            absent: Some(false),
            short: self.short,
            long: self.long,
            help: self.help,
        }
    }
    pub fn req_switch(self) -> Flag<bool> {
        Flag {
            present: true,
            absent: None,
            short: self.short,
            long: self.long,
            help: self.help,
        }
    }

    /// present/absent value flag
    pub fn flag<T>(self, present: T, absent: T) -> Flag<T> {
        Flag {
            present,
            absent: Some(absent),
            short: self.short,
            long: self.long,
            help: self.help,
        }
    }

    /// required flag
    pub fn req_flag<T>(self, present: T) -> Flag<T> {
        Flag {
            present,
            absent: None,
            short: self.short,
            long: self.long,
            help: self.help,
        }
    }

    pub fn argument(self, metavar: &'static str) -> Argument {
        Argument {
            short: self.short,
            long: self.long,
            help: self.help,
            metavar,
        }
    }
}

pub fn command<T, M>(name: &'static str, help: M, p: ParserInfo<T>) -> Parser<T>
where
    T: 'static,
    M: Into<String>,
{
    let parse = move |mut i: Args| match i.take_word(name) {
        Some(i) => (p.parse)(i),
        None => Err(Error::Stderr(format!("expected {}", name))),
    };
    let meta = Meta::from(Item {
        short: None,
        long: Some(name),
        metavar: None,
        help: Some(help.into()),
        kind: ItemKind::Command,
    });
    Parser {
        parse: Rc::new(parse),
        meta,
    }
}

#[derive(Default)]
pub struct Flag<T> {
    present: T,
    absent: Option<T>,
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
}

impl<T> Flag<T> {
    pub fn build(self) -> Parser<T>
    where
        T: Clone + 'static,
    {
        let item = Item {
            short: self.short.first().copied(),
            long: self.long.first().copied(),
            metavar: None,
            help: self.help,
            kind: ItemKind::Flag,
        };
        let required = self.absent.is_none();
        let meta = item.required(required);

        let missing = if required {
            Error::Missing(vec![meta.clone()])
        } else {
            Error::Stdout(String::new())
        };

        let parse = move |mut i: Args| {
            for &short in self.short.iter() {
                if let Some(i) = i.take_short_flag(short) {
                    return Ok((self.present.clone(), i));
                }
            }
            for long in self.long.iter() {
                if let Some(i) = i.take_long_flag(long) {
                    return Ok((self.present.clone(), i));
                }
            }
            Ok((
                self.absent.as_ref().ok_or_else(|| missing.clone())?.clone(),
                i,
            ))
        };
        Parser {
            parse: Rc::new(parse),
            meta,
        }
    }
}

impl<T> Flag<T> {
    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}

pub struct Argument {
    short: Vec<char>,
    long: Vec<&'static str>,
    help: Option<String>,
    metavar: &'static str,
}

impl Argument {
    fn build_both(self) -> Parser<Word> {
        let item = Item {
            kind: ItemKind::Flag,
            short: self.short.first().copied(),
            long: self.long.first().copied(),
            metavar: Some(self.metavar),
            help: self.help,
        };
        let meta = item.required(true);
        let meta2 = meta.clone();
        let parse = move |mut i: Args| {
            for &short in self.short.iter() {
                if let Some((w, c)) = i.take_short_arg(short)? {
                    return Ok((w, c));
                }
            }
            for long in self.long.iter() {
                if let Some((w, c)) = i.take_long_arg(long)? {
                    return Ok((w, c));
                }
            }
            Err(Error::Missing(vec![meta2.clone()]))
        };

        Parser {
            parse: Rc::new(parse),
            meta,
        }
    }

    pub fn build(self) -> Parser<String> {
        self.build_both().parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
    }

    pub fn build_os(self) -> Parser<OsString> {
        self.build_both().map(|x| x.os)
    }

    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}

pub struct Positional {
    help: Option<String>,
    metavar: &'static str,
}

pub fn positional(metavar: &'static str) -> Positional {
    Positional {
        metavar,
        help: None,
    }
}

impl Positional {
    fn build_both(self) -> Parser<Word> {
        let item = Item {
            short: None,
            long: None,
            metavar: Some(self.metavar),
            help: self.help,
            kind: ItemKind::Positional,
        };
        let meta = item.required(true);
        let meta2 = meta.clone();

        let parse = move |mut args: Args| match args.take_positional() {
            Some((word, args)) => return Ok((word, args)),
            None => Err(Error::Missing(vec![meta2.clone()])),
        };
        Parser {
            parse: Rc::new(parse),
            meta,
        }
    }

    pub fn build(self) -> Parser<String> {
        self.build_both().parse(|x| x.utf8.ok_or("not utf8")) // TODO - provide a better diagnostic
    }

    pub fn build_os(self) -> Parser<OsString> {
        self.build_both().map(|x| x.os)
    }

    pub fn help<M>(mut self, help: M) -> Self
    where
        M: Into<String>,
    {
        self.help = Some(help.into());
        self
    }
}
