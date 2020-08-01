//! Parse text into words, newlines and whitespace sequences.
//!
//! ```rust
//! use embedded_text::parser::{Parser, Token};
//!
//! let parser = Parser::parse("Hello, world!\n");
//! let tokens = parser.collect::<Vec<Token<'_>>>();
//!
//! assert_eq!(
//!     vec![
//!         Token::Word("Hello,"),
//!         Token::Whitespace(1),
//!         Token::Word("world!"),
//!         Token::NewLine
//!     ],
//!     tokens
//! );
//! ```
use core::str::Chars;

/// A text token
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    /// A newline character.
    NewLine,

    /// A \r character.
    CarriageReturn,

    /// A number of whitespace characters.
    Whitespace(u32),

    /// A word (a sequence of non-whitespace characters).
    Word(&'a str),
}

/// Text parser. Turns a string into a stream of [`Token`] objects.
///
/// [`Token`]: enum.Token.html
#[derive(Clone, Debug)]
pub struct Parser<'a> {
    inner: Chars<'a>,
}

impl<'a> Parser<'a> {
    /// Create a new parser object to process the given piece of text.
    #[inline]
    #[must_use]
    pub fn parse(text: &'a str) -> Self {
        Self {
            inner: text.chars(),
        }
    }

    /// Returns the next token without advancing.
    #[inline]
    #[must_use]
    pub fn peek(&self) -> Option<Token> {
        self.clone().next()
    }

    /// Returns true if there are no tokens to process.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.as_str().is_empty()
    }

    /// Returns the number of unprocessed bytes.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.inner.as_str().len()
    }

    fn is_word_char(c: char) -> bool {
        !c.is_whitespace() || c == '\u{A0}'
    }

    fn is_space_char(c: char) -> bool {
        c.is_whitespace() && !['\n', '\r', '\u{A0}'].contains(&c)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Token<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let string = self.inner.as_str();
        self.inner.next().map(|c| {
            let mut iter = self.inner.clone();
            match c {
                '\n' => Token::NewLine,
                '\r' => Token::CarriageReturn,

                c if Self::is_space_char(c) => {
                    let mut len = 1;
                    while let Some(c) = iter.next() {
                        if Self::is_space_char(c) {
                            len += 1;
                            self.inner = iter.clone();
                        } else {
                            // consume the whitespaces
                            return Token::Whitespace(len);
                        }
                    }

                    // consume all the text
                    self.inner = "".chars();
                    Token::Whitespace(len)
                }

                _ => {
                    while let Some(c) = iter.next() {
                        if !Self::is_word_char(c) {
                            let offset = string.len() - self.inner.as_str().len();
                            return Token::Word(unsafe {
                                // don't worry
                                string.get_unchecked(0..offset)
                            });
                        } else {
                            self.inner = iter.clone();
                        }
                    }

                    // consume all the text
                    self.inner = "".chars();
                    Token::Word(&string)
                }
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Parser, Token};
    #[test]
    fn parse() {
        // (At least) for now, \r is considered a whitespace
        let text = "Lorem ipsum \r dolor sit amet, conse😅ctetur adipiscing\nelit";

        assert_eq!(
            Parser::parse(text).collect::<Vec<Token>>(),
            vec![
                Token::Word("Lorem"),
                Token::Whitespace(1),
                Token::Word("ipsum"),
                Token::Whitespace(1),
                Token::CarriageReturn,
                Token::Whitespace(1),
                Token::Word("dolor"),
                Token::Whitespace(1),
                Token::Word("sit"),
                Token::Whitespace(1),
                Token::Word("amet,"),
                Token::Whitespace(1),
                Token::Word("conse😅ctetur"),
                Token::Whitespace(1),
                Token::Word("adipiscing"),
                Token::NewLine,
                Token::Word("elit"),
            ]
        );
    }

    #[test]
    fn parse_multibyte_last() {
        // (At least) for now, \r is considered a whitespace
        let text = "test😅";

        assert_eq!(
            Parser::parse(text).collect::<Vec<Token>>(),
            vec![Token::Word("test😅"),]
        );
    }

    #[test]
    fn parse_nbsp_as_word_char() {
        let text = "test\u{A0}word";

        assert_eq!(9, "test\u{A0}word".chars().count());
        assert_eq!(
            Parser::parse(text).collect::<Vec<Token>>(),
            vec![Token::Word("test\u{A0}word"),]
        );
        assert_eq!(
            Parser::parse(" \u{A0}word").collect::<Vec<Token>>(),
            vec![Token::Whitespace(1), Token::Word("\u{A0}word"),]
        );
    }
}
