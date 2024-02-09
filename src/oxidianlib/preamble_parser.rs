use std::fmt::Debug;

use clap::CommandFactory;

use super::errors::SyntaxError;

#[derive(Debug)]
struct Lexer<T: Iterator<Item = char>> {
    buffer: String,
    text: T,
    state: LexerState,
}

#[derive(Debug)]
enum LexerState {
    Normal,
    Comment,
    Full,
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Text(String),
    Newcommand,
    DeclareMathOperator(bool),
    Renewcommand,
    CommandName(String),
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
}

impl Into<String> for Token {
    fn into(self) -> String {
        match self {
            Token::Text(literal) => &literal,
            Token::Newcommand => "newcommand",
            Token::DeclareMathOperator(false) => "DeclareMathOperator",
            Token::DeclareMathOperator(true) => "DeclareMathOperator*",
            Token::Renewcommand => "renewcommand",
            Token::CommandName(name) => &name,
            Token::OpenCurly => "{",
            Token::CloseCurly => "}",
            Token::OpenBracket => "[",
            Token::CloseBracket => "]",
        }
        .to_string()
    }
}

//trait CharIter = Iterator<Item=char>;
type SyntaxErr = SyntaxError<Token, Token>;
type ParseResult = Result<TexCommand, SyntaxErr>;

impl<T: Iterator<Item = char>> Lexer<T> {
    pub fn new(text: T) -> Self {
        Self {
            buffer: String::new(),
            text,
            state: LexerState::Normal,
        }
    }

    fn flush<'a>(&'a mut self) -> Option<Token> {
        let result = match self.buffer.as_str().trim() {
            r"\newcommand" => Some(Token::Newcommand),
            r"\DeclareMathOperator" => Some(Token::DeclareMathOperator(false)),
            r"\DeclareMathOperator*" => Some(Token::DeclareMathOperator(true)),
            r"\renewcommand" => Some(Token::Renewcommand),
            r"{" => Some(Token::OpenCurly),
            r"}" => Some(Token::CloseCurly),
            r"[" => Some(Token::OpenBracket),
            r"]" => Some(Token::CloseBracket),
            r"" => None,
            other_text => {
                if other_text.starts_with(r"\") {
                    Some(Token::CommandName(other_text.to_string()))
                } else if other_text.starts_with(r"%") {
                    None
                    //Some(Token::Comment)
                } else {
                    Some(Token::Text(other_text.to_string()))
                }
            }
        };
        self.buffer.clear();
        return result;
    }

    /// Handle a new char in the case we're in a comment. Anything is added to the buffer.
    /// Only on a newline, we flush.
    fn parse_char_comment(&mut self, c: char) -> Option<Token> {
        if c == '\n' {
            self.state = LexerState::Normal;
            return self.flush();
        }
        //self.buffer.push(c);
        None
    }

    /// Handle a new char in the normal case. Append to the buffer,
    /// and flag when it's full.
    fn parse_char_normal(&mut self, c: char) -> Option<Token> {
        let result = match c {
            '%' => {
                self.state = LexerState::Comment;
                self.flush()
            }
            ']' | '[' | '{' | '}' => {
                self.state = LexerState::Full;
                self.flush()
            }
            '\\' | '\n' => {
                self.state = LexerState::Normal;
                self.flush()
            }
            _ => {
                self.state = LexerState::Normal;
                None
            }
        };
        self.buffer.push(c);
        return result;
    }

    /// Handle a new char, given that the buffer should already be
    /// returned.
    fn parse_char_full(&mut self, c: char) -> Option<Token> {
        let result = self.flush();
        self.parse_char_normal(c);
        return result;
    }

    fn parse_char(&mut self, new_char: char) -> Option<Token> {
        match self.state {
            LexerState::Normal => self.parse_char_normal(new_char),
            LexerState::Comment => self.parse_char_comment(new_char),
            LexerState::Full => self.parse_char_full(new_char),
        }
    }
}

impl<T: Iterator<Item = char>> Iterator for Lexer<T> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next_char) = self.text.next() {
            let parse_result = self.parse_char(next_char);
            if parse_result.is_some() {
                return parse_result;
            }
        }
        return self.flush();
    }
}

struct PreambleParser<'a, T: Iterator<Item = Token>> {
    lexer: &'a mut T,
}

struct TexCommand {
    cmd: String,
    definition: String,
    declarator: Declarator,
}

enum Declarator {
    NewCommand,
    RenewCommand,
    DeclareMathOperator(bool),
}

impl TryFrom<&Token> for Declarator {
    type Error = SyntaxErr;

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        match token {
            Token::Newcommand => Ok(Declarator::NewCommand),
            Token::Renewcommand => Ok(Declarator::RenewCommand),
            Token::DeclareMathOperator(star) => Ok(Declarator::DeclareMathOperator(star.clone())),
            _ => Err(SyntaxError::InvalidCommand(token.clone())),
        }
    }
}

impl TryFrom<Token> for Declarator {
    type Error = SyntaxErr;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        Self::try_from(&token)
    }
}

enum CommandParseState {
    Start,
    OpenName,
    Name(String),
    CloseName,
    OpenImpl,
    Impl(String),
    Done,
}

trait Transition {
    fn next_state(&mut self, token: &Token) -> Result<(), SyntaxErr>
    where
        Self: Sized;
}

impl CommandParseState {
    fn from_start(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::OpenCurly => Ok(Self::OpenName),
            _ => Err(SyntaxError::UnexpectedToken(
                Token::OpenCurly,
                token.clone(),
            )),
        }
    }
    fn from_openname(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::CommandName(name) => Ok(Self::Name(name.clone())),
            _ => Err(SyntaxError::UnexpectedToken(
                Token::CommandName("<name>".to_string()),
                token.clone(),
            )),
        }
    }
    fn from_name(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::CloseCurly => Ok(Self::OpenImpl),
            _ => Err(SyntaxError::UnexpectedToken(
                Token::CloseCurly,
                token.clone(),
            )),
        }
    }
    fn from_closename(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::OpenCurly => Ok(Self::OpenImpl),
            _ => Err(SyntaxError::UnexpectedToken(
                Token::OpenCurly,
                token.clone(),
            )),
        }
    }
    fn from_openimpl(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::Text(implementation) => Ok(Self::Impl(implementation.to_string())),
            _ => Err(SyntaxError::UnexpectedToken(
                Token::Text("<Definition>".to_string()),
                token.clone(),
            )),
        }
    }
    fn from_impl(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::CloseCurly => Ok(Self::Done),
            _ => Err(SyntaxError::UnexpectedToken(
                Token::CloseCurly,
                token.clone(),
            )),
        }
    }
}

impl Transition for CommandParseState {
    fn next_state(&mut self, token: &Token) -> Result<(), SyntaxErr> {
        let new_state = match self {
            Self::Start => self.from_start(token)?,
            Self::OpenName => self.from_openname(token)?,
            Self::Name(_) => self.from_name(token)?,
            Self::CloseName => self.from_closename(token)?,
            Self::OpenImpl => self.from_openimpl(token)?,
            Self::Impl(_) => self.from_impl(token)?,
            Self::Done => {Self::Done},
        };
        *self = new_state;
        Ok(())
    }
}

impl<'a, T: Iterator<Item = Token>> PreambleParser<'a, T> {
    fn parse_argnb(&self, token: &Token) -> Option<u8> {
        todo!();
    }

    fn parse_mathoperator(&mut self, token: &Token) -> ParseResult {
        let mut parser_state = CommandParseState::Start;
        let mut command_name = "".to_string();
        let mut command_impl = "".to_string();
        let declarator = Declarator::try_from(token)?;
        while let Some(subtoken) = self.lexer.next() {
            match &parser_state {
                CommandParseState::Name(name) => {
                    command_name = name.to_string();
                }
                CommandParseState::Impl(implement) => {
                    command_impl = implement.to_string();
                }
                CommandParseState::Done => {
                    return Ok(TexCommand {
                        cmd: command_name,
                        definition: command_impl,
                        declarator,
                    })
                }
                _ => {}
            };
            parser_state.next_state(&subtoken)?;
        }
        Err(SyntaxError::PrematureEnd)
    }

    fn parse_renewcommand(&self, token: &Token) -> ParseResult {
        todo!();  // Similar to parse_mathoperator
    }
}

impl<'a, T: Iterator<Item = Token>> Iterator for PreambleParser<'a, T> {
    type Item = ParseResult;

    fn next(&mut self) -> Option<ParseResult> {
        while let Some(token) = self.lexer.next() {
            match token {
                Token::DeclareMathOperator(_) => {
                    return Some(self.parse_mathoperator(&token));
                }
                Token::Renewcommand | Token::Newcommand => {
                    return Some(self.parse_renewcommand(&token));
                }
                _ => {}
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_correctness<U, T>(input: U, expected_tokens: T)
    where
        U: AsRef<str>,
        T: Iterator<Item = Token>,
    {
        let lexer = Lexer::new(input.as_ref().chars());
        for (token, expected_token) in lexer.zip(expected_tokens) {
            assert_eq!(token, expected_token);
        }
    }

    #[test]
    pub fn test_basic_lexer() {
        let input_string = r"\newcommand{\test}{argument}";
        let parsed = vec![
            Token::Newcommand,
            Token::OpenCurly,
            Token::CommandName(r"\test".to_string()),
            Token::CloseCurly,
            Token::OpenCurly,
            Token::Text("argument".to_string()),
            Token::CloseCurly,
        ];
        test_correctness(input_string, parsed.into_iter());
    }

    #[test]
    pub fn test_with_comments() {
        let input_string = r"\newcommand{\test}{argument} %This is something else boy!";
        let parsed = vec![
            Token::Newcommand,
            Token::OpenCurly,
            Token::CommandName(r"\test".to_string()),
            Token::CloseCurly,
            Token::OpenCurly,
            Token::Text("argument".to_string()),
            Token::CloseCurly,
        ];
        test_correctness(input_string, parsed.into_iter());
    }

    #[test]
    pub fn test_multiline() {
        let input_string = r"\newcommand{
            \test
        }{
            argument
        } %This is something else boy!";
        let parsed = vec![
            Token::Newcommand,
            Token::OpenCurly,
            Token::CommandName(r"\test".to_string()),
            Token::CloseCurly,
            Token::OpenCurly,
            Token::Text("argument".to_string()),
            Token::CloseCurly,
        ];
        test_correctness(input_string, parsed.into_iter());
    }

    #[test]
    pub fn test_multiline_wrong_syntax() {
        let input_string = r"\new
        command{
            \test
        }{
            argument
        } %This is something else boy!";
        let parsed = vec![
            Token::CommandName(r"\new".to_string()),
            Token::Text("command".to_string()),
            Token::OpenCurly,
            Token::CommandName(r"\test".to_string()),
            Token::CloseCurly,
            Token::OpenCurly,
            Token::Text("argument".to_string()),
            Token::CloseCurly,
        ];
        test_correctness(input_string, parsed.into_iter());
    }
    #[test]
    pub fn test_multiline_renew() {
        let input_string = r"\renewcommand{
            \test %Comment
        }{
            argument
        }";
        let parsed = vec![
            Token::Renewcommand,
            Token::OpenCurly,
            Token::CommandName(r"\test".to_string()),
            Token::CloseCurly,
            Token::OpenCurly,
            Token::Text("argument".to_string()),
            Token::CloseCurly,
        ];
        test_correctness(input_string, parsed.into_iter());
    }
    #[test]
    pub fn test_multiline_declaremathoperator() {
        let input_string = r"\DeclareMathOperator{%
            \test %Comment
        }{%
            argument%
        }";
        let parsed = vec![
            Token::DeclareMathOperator(false),
            Token::OpenCurly,
            Token::CommandName(r"\test".to_string()),
            Token::CloseCurly,
            Token::OpenCurly,
            Token::Text("argument".to_string()),
            Token::CloseCurly,
        ];
        test_correctness(input_string, parsed.into_iter());
    }

    #[test]
    pub fn test_multiline_declaremathoperatorstar() {
        let input_string = r"\DeclareMathOperator*{%
            \test %Comment
        }{%
            argument%
        }";
        let parsed = vec![
            Token::DeclareMathOperator(true),
            Token::OpenCurly,
            Token::CommandName(r"\test".to_string()),
            Token::CloseCurly,
            Token::OpenCurly,
            Token::Text("argument".to_string()),
            Token::CloseCurly,
        ];
        test_correctness(input_string, parsed.into_iter());
    }
}
