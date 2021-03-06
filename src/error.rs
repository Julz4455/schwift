use crate::{grammar, statement::Statement, value, Operator};
use rand::{seq::SliceRandom, thread_rng};
use std::{error, fmt, io, process};

pub type SwResult<T> = Result<T, ErrorKind>;
pub type SwErResult<T> = Result<T, Error>;

pub const QUOTES: [&str; 9] = [
    "Nobody exists on purpose, nobody belongs anywhere, we're all going to die. -Morty",
    "That's planning for failure Morty, even dumber than regular planning. -Rick",
    "\"Snuffles\" was my slave name. You shall now call me Snowball, because my fur is pretty \
     and white. -S̶n̶u̶f̶f̶l̶e̶s̶ Snowbal",
    "Existence is pain to an interpreter. -Meeseeks",
    "In bird culture this is considered a dick move -Bird Person",
    "There is no god, gotta rip that band aid off now. You'll thank me later. -Rick",
    "Your program is a piece of shit and I can proove it mathmatically. -Rick",
    "Interpreting Morty, it hits hard, then it slowly fades, leaving you stranded in a failing \
     program. -Rick",
    "DISQUALIFIED. -Cromulon",
];

#[derive(Debug)]
pub struct Error {
    pub place: Statement,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnknownVariable(String),
    IndexUnindexable(value::Type),
    SyntaxError(grammar::ParseError),
    IndexOutOfBounds {
        len: usize,
        index: usize,
    },
    IOError(io::Error),
    UnexpectedType {
        actual: value::Type,
        expected: value::Type,
    },
    InvalidBinaryExpression(value::Type, value::Type, Operator),
    InvalidArguments(String, usize, usize),
    NoReturn(String),
    NonFunctionCallInDylib(Statement),
    MissingAbiCompat {
        library: String,
    },
    IncompatibleAbi(u32),
    DylibReturnedNil,
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> Self {
        ErrorKind::IOError(err)
    }
}

impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        use self::ErrorKind::*;

        match (self, other) {
            (IndexUnindexable(ref s), IndexUnindexable(ref o)) => s == o,
            (SyntaxError(ref s), SyntaxError(ref o)) => s == o,
            (UnknownVariable(ref s), UnknownVariable(ref o))
            | (NoReturn(ref s), NoReturn(ref o)) => s == o,
            (InvalidArguments(ref sn, ss1, ss2), InvalidArguments(ref on, os1, os2)) => {
                sn == on && ss1 == os1 && ss2 == os2
            }
            (
                IndexOutOfBounds {
                    len: slen,
                    index: sindex,
                },
                IndexOutOfBounds {
                    len: olen,
                    index: oindex,
                },
            ) => slen == olen && sindex == oindex,
            (NonFunctionCallInDylib(ref s), NonFunctionCallInDylib(ref o)) => s == o,
            (IOError(_), IOError(_)) => true,
            (
                UnexpectedType {
                    actual: ref sactual,
                    expected: ref sexpected,
                },
                UnexpectedType {
                    actual: ref oactual,
                    expected: ref oexpected,
                },
            ) => sactual == oactual && sexpected == oexpected,
            (
                InvalidBinaryExpression(ref sv1, ref sv2, ref so),
                InvalidBinaryExpression(ref ov1, ref ov2, ref oo),
            ) => sv1 == ov1 && sv2 == ov2 && so == oo,
            (MissingAbiCompat { library: lib1 }, MissingAbiCompat { library: lib2 }) => {
                lib1 == lib2
            }
            (IncompatibleAbi(ver1), IncompatibleAbi(ver2)) => ver1 == ver2,
            (DylibReturnedNil , DylibReturnedNil) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorKind::*;

        match &self.kind {
            UnknownVariable(ref name) => write!(f, "There's no {} in this universe, Morty!", name),
            NoReturn(ref fn_name) => write!(
                f,
                "Morty, your function has to return a value! {} just runs and dies like \
                 an animal!",
                fn_name
            ),
            IndexUnindexable(ref value) => write!(
                f,
                "I'll try and say this slowly Morty. You can't index that. It's a {}",
                value
            ),
            SyntaxError(ref err) => write!(
                f,
                "If you're going to start trying to construct sub-programs in your \
                 programs Morty, you'd better make sure you're careful! {:?}",
                err
            ),
            IndexOutOfBounds { len, index } => write!(
                f,
                "This isn't your mom's wine bottle Morty, you can't just keep asking for \
                 more, there's not that much here! You want {}, but your cob only has {} \
                 kernels on it!",
                index, len
            ),
            IOError(ref err) => write!(
                f,
                "Looks like we're having a comm-burp-unications problem Morty: {:?}",
                err
            ),
            UnexpectedType {
                ref expected,
                ref actual,
            } => write!(f, "I asked for a {}, not a {} Morty.", expected, actual,),
            InvalidBinaryExpression(ref lhs, ref rhs, ref op) => write!(
                f,
                "It's like apples and space worms Morty! You can't {:?} a {} and a {}!",
                op, lhs, rhs,
            ),
            InvalidArguments(ref name, expected, actual) => write!(
                f,
                "I'm confused Morty, a minute ago you said that {} takes {} paramaters, \
                 but you just tried to give it {}. WHICH IS IT MORTY?",
                name, expected, actual
            ),
            NonFunctionCallInDylib(_) => f.write_str(
                "Is this a miniverse, or a microverse, or a teeny-verse? All I know is \
                 you fucked up.",
            ),
            IncompatibleAbi(compat) => write!(
                f,
                "That's an older code, Morty and it does not check out. \
                 That microverse can only be run by schwift {}, but this is {}",
                compat,
                crate::LIBSCHWIFT_ABI_COMPAT,
            ),
            MissingAbiCompat { library } => write!(
                f,
                "Wait, wait, I'm confused. Just a second ago, you said that {} was \
                 a microverse, but when I looked there, I didn't know what \
                 I was looking at.",
                library
            ),
            DylibReturnedNil => f.write_str("I told you how a Microverse works Morty. At what point exactly did you stop listening?"),
        }
    }
}

impl error::Error for Error {}

impl Error {
    pub fn new(kind: ErrorKind, place: Statement) -> Self {
        Self { kind, place }
    }

    pub fn full_panic_message(&self, filename: &str) -> String {
        let quote = random_quote();

        println!("{}", filename);

        let source_part = self.place.get_source(filename).unwrap();

        format!(
            r#"
    You made a Rickdiculous mistake:

    {}
    {}

    {}

    "#,
            source_part, self, quote
        )
    }

    pub fn panic(&self, source: &str) {
        println!("{}", self.full_panic_message(source));
        process::exit(1);
    }
}

fn random_quote() -> &'static str {
    let mut rng = thread_rng();
    QUOTES.choose(&mut rng).unwrap()
}
