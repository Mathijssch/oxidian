use super::errors::SyntaxError;
use super::lexer::{SyntaxErr, Token, Lexer};

type ParseResult = Result<TexCommand, SyntaxErr>;

struct PreambleParser<T: Iterator<Item = Token>> {
    lexer: T,
}

impl<T: Iterator<Item = Token>> PreambleParser<T> {
    pub fn new(lexer: T) -> Self {
        Self { lexer }
    }
}

#[derive(PartialEq, Debug)]
pub struct TexCommand {
    pub cmd: String,
    pub definition: String,
    pub declarator: Declarator,
    pub argc: Option<u8>,
    pub default_args: Option<String>,
}



impl TexCommand {
    
    pub fn new<C: Into<String>, D: Into<String>>(
        cmd: C,
        definition: D,
        declarator: Declarator,
    ) -> Self {
        Self {  
            cmd: cmd.into(),
            definition: definition.into(),
            declarator,
            argc: None,
            default_args: None,
        }
    }

    #[allow(dead_code)]  // Useful for testing purposes.
    pub fn with_args(mut self, count: u8) -> Self {
        self.argc = Some(count);
        return self
    }

    #[allow(dead_code)]  // Useful for testing purposes.
    pub fn with_defaults<T: Into<String>>(mut self, count: u8, defaults: T) -> Self {
        self.argc = Some(count);
        self.default_args = Some(defaults.into());
        return self
    }

    #[allow(dead_code)]  // Useful for testing purposes.
    pub fn newcommand<C: Into<String>, D: Into<String>>(cmd: C, definition: D) -> Self {
        Self::new(cmd, definition, Declarator::NewCommand)
    }

    #[allow(dead_code)]  // Useful for testing purposes.
    pub fn declare_math_operator<C: Into<String>, D: Into<String>>(cmd: C, definition: D) -> Self {
        Self::new(cmd, definition, Declarator::DeclareMathOperator(false))
    }

    #[allow(dead_code)]  // Useful for testing purposes.
    pub fn renewcommand<C: Into<String>, D: Into<String>>(cmd: C, definition: D) -> Self {
        Self::new(cmd, definition, Declarator::RenewCommand)
    }
}

#[derive(PartialEq, Debug)]
pub enum Declarator {
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

#[derive(Debug)]
enum CommandParseState {
    Start,
    OpenName,
    Name(String, bool),
    CloseName,
    OpenArgCount,
    ArgCount(u8),
    CloseArgCount,
    OpenDefaultArgs,
    DefaultArgs(String),
    CloseDefaultArgs,
    OpenImpl,
    Impl(String, u8),
    Done,
}

trait Transition {
    fn next_state(&mut self, token: &Token) -> Result<(), SyntaxErr>
    where
        Self: Sized;
}

impl CommandParseState {

    fn get_curly_depth(&self) -> u8 {
        match self {
            Self::Impl(_, count) => *count, 
            _ => 0
        }
    }

    fn from_start(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::OpenCurly => Ok(Self::OpenName),
            Token::CommandName(name) => Ok(Self::Name(name.clone(), false)),
            _ => Err(SyntaxError::UnexpectedToken(
                vec![Token::OpenCurly, Token::CommandName("<cmd>".to_string())],
                token.clone(),
            )),
        }
    }
    fn from_openname(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::CommandName(name) => Ok(Self::Name(name.clone(), true)),
            _ => Err(SyntaxError::UnexpectedToken(
                vec![Token::CommandName("<name>".to_string())],
                token.clone(),
            )),
        }
    }
    fn from_name(&self, token: &Token, expect_close: bool) -> Result<Self, SyntaxErr> {
        if expect_close {
            match token {
                Token::CloseCurly => Ok(Self::CloseName),
                _ => Err(SyntaxError::UnexpectedToken(
                    vec![Token::CloseCurly],
                    token.clone(),
                )),
            }
        } else {
            self.from_closename(token)
        }
    }
    fn from_closename(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::OpenCurly => Ok(Self::OpenImpl),
            Token::OpenBracket => Ok(Self::OpenArgCount),
            _ => Err(SyntaxError::UnexpectedToken(
                vec![Token::OpenCurly, Token::OpenBracket],
                token.clone(),
            )),
        }
    }
    fn from_argcount(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::CloseBracket => Ok(Self::CloseArgCount),
            _ => Err(SyntaxError::UnexpectedToken(
                vec![Token::CloseBracket],
                token.clone(),
            )),
        }
    }
    fn from_closeargcount(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::OpenCurly => Ok(Self::OpenImpl),
            Token::OpenBracket => Ok(Self::OpenDefaultArgs),
            _ => Err(SyntaxError::UnexpectedToken(
                vec![Token::OpenCurly, Token::OpenBracket],
                token.clone(),
            )),
        }
    }
    fn from_openargcount(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::Text(number) => Ok(Self::ArgCount(
                number
                    .parse::<u8>()
                    .map_err(|_| SyntaxErr::InvalidNumber(token.clone()))?,
            )),
            Token::CloseBracket => Ok(Self::CloseArgCount),
            //Token::OpenBracket => Ok(Self::OpenArgCount),
            _ => Err(SyntaxError::UnexpectedToken( 
                    vec![Token::Text("<Argument count>".to_string()),
                         Token::CloseBracket],
                token.clone(),
            )),
        }
    }
    fn from_defaultargs(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::CloseBracket => Ok(Self::CloseArgCount),
            _ => Err(SyntaxError::UnexpectedToken(
                vec![Token::CloseBracket],
                token.clone(),
            )),
        }
    }
    fn from_closedefaultargs(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::OpenCurly => Ok(Self::OpenImpl),
            _ => Err(SyntaxError::UnexpectedToken(
                vec![Token::OpenCurly],
                token.clone(),
            )),
        }
    }
    fn from_opendefaultargs(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::Text(defaults) => Ok(Self::DefaultArgs(defaults.to_string())),
            Token::CloseBracket => Ok(Self::CloseDefaultArgs),
            Token::OpenBracket => Ok(Self::OpenArgCount),
            _ => Err(SyntaxError::UnexpectedToken(
                vec![Token::Text("<Default args>".to_string()),
                Token::CloseBracket,
                Token::OpenBracket
                ],
                token.clone(),
            )),
        }
    }
    fn from_openimpl(&self, token: &Token) -> Result<Self, SyntaxErr> {
        match token {
            Token::CloseCurly => Ok(Self::Done),
            Token::OpenCurly => Ok(Self::Impl(token.into(), 1)),
            _ => {Ok(Self::Impl(token.into(), 0))}
            //Token::Text(implementation) => Ok(Self::Impl(implementation.to_string())),
            //_ => Err(SyntaxError::UnexpectedToken(
            //    Token::Text("<Definition>".to_string()),
            //    token.clone(),
            //)),
        }
    }
    fn from_impl(&self, token: &Token) -> Result<Self, SyntaxErr> {
        //println!("Getting token {:?}", token);
        let depth = self.get_curly_depth();
        match token {
            Token::CloseCurly => { 
                if depth > 0 {
                    //println!("Moving to depth {}", depth - 1);
                    return Ok(Self::Impl(token.into(), depth - 1))
                } else {
                    //println!("Done!");
                    return Ok(Self::Done)
                }
            }, 
            Token::OpenCurly => { 
                //println!("Moving to depth {}", depth + 1);
                Ok(Self::Impl(token.into(), depth + 1)) 
            }
            _ => {Ok(Self::Impl(token.into(), depth))}
            //Token::CloseCurly => Ok(Self::Done),
            //_ => Err(SyntaxError::UnexpectedToken(
            //    Token::CloseCurly,
            //    token.clone(),
            //)),
        }
    }
}

impl Transition for CommandParseState {
    fn next_state(&mut self, token: &Token) -> Result<(), SyntaxErr> {
        let new_state = match &self {
            Self::Start => self.from_start(token)?,
            Self::OpenName => self.from_openname(token)?,
            Self::Name(_, with_bracket) => self.from_name(token, *with_bracket)?,
            Self::CloseName => self.from_closename(token)?,
            Self::OpenArgCount => self.from_openargcount(token)?,
            Self::ArgCount(_) => self.from_argcount(token)?,
            Self::CloseArgCount => self.from_closeargcount(token)?,
            Self::OpenDefaultArgs => self.from_opendefaultargs(token)?,
            Self::DefaultArgs(_) => self.from_defaultargs(token)?,
            Self::CloseDefaultArgs => self.from_closedefaultargs(token)?,
            Self::OpenImpl => self.from_openimpl(token)?,
            Self::Impl(_,_) => self.from_impl(token)?,
            Self::Done => Self::Done,
        };
        *self = new_state;
        Ok(())
    }
}

impl<T: Iterator<Item = Token>> PreambleParser<T> {
    fn parse_mathoperator(&mut self, token: &Token) -> ParseResult {
        let mut parser_state = CommandParseState::Start;
        let mut command_name = "".to_string();
        let mut command_impl = "".to_string();
        let declarator = Declarator::try_from(token)?;

        while let Some(subtoken) = self.lexer.next() {
            match &parser_state {
                CommandParseState::OpenArgCount
                | CommandParseState::ArgCount(_)
                | CommandParseState::CloseArgCount
                | CommandParseState::OpenDefaultArgs
                | CommandParseState::DefaultArgs(_)
                | CommandParseState::CloseDefaultArgs => {
                    return Err(SyntaxErr::NoArguments);
                }
                CommandParseState::Name(name, _) => {
                    command_name = name.to_string();
                }
                CommandParseState::Impl(implement, _) => {
                    command_impl.push_str(implement);
                }
                CommandParseState::Done => {
                    return Ok(TexCommand {
                        cmd: command_name,
                        definition: command_impl,
                        declarator,
                        argc: None,
                        default_args: None,
                    })
                }
                _ => {}
            };
            parser_state.next_state(&subtoken)?;
        }
        Err(SyntaxError::PrematureEnd)
    }

    fn parse_renewcommand(&mut self, token: &Token) -> ParseResult {
        let mut parser_state = CommandParseState::Start;
        let mut command_name = "".to_string();
        let mut argc: Option<u8> = None;
        let mut default_args: Option<String> = None;
        let mut command_impl = "".to_string();
        let declarator = Declarator::try_from(token)?;
        while let Some(subtoken) = self.lexer.next() {
            parser_state.next_state(&subtoken)?;
            //println!("Current impl: {}", command_impl);
            //println!("Current state: {:?}", &parser_state);
            //println!("Handling {:?}", subtoken);
            match &parser_state {
                CommandParseState::Name(name, _) => {
                    command_name = name.to_string();
                }
                CommandParseState::Impl(implement, _) => {
                    command_impl.push_str(implement);
                }
                CommandParseState::ArgCount(nb) => argc = Some(*nb),
                CommandParseState::DefaultArgs(defaults) => {
                    default_args = Some(defaults.to_string());
                }
                CommandParseState::Done => {
                    return Ok(TexCommand {
                        cmd: command_name,
                        definition: command_impl,
                        declarator,
                        argc,
                        default_args,
                    })
                }
                _ => {}
            };
        }
        Err(SyntaxError::PrematureEnd)
    }
}

impl<'a, T: Iterator<Item = Token>> Iterator for PreambleParser<T> {
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

pub fn parse_preamble<'a>(preamble: &'a str) -> impl Iterator<Item=ParseResult> + 'a {
    let lexer = Lexer::new(preamble.chars());
    PreambleParser::new(lexer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_valid(input_string: &str, expected: &[TexCommand]) {
        for (result, correct) in parse_preamble(input_string).zip(expected) {
            if let Ok(res) = result { 
                assert_eq!(&res, correct);
            } else {
            panic!("{:?}", result.unwrap_err());
            }
        }
    }

    fn test_invalid(input_string: &str) {
        for result in parse_preamble(input_string) {
            assert!(result.is_err())
        }
    }


    #[test]
    fn basic_newcommand() {
        test_valid(r"\newcommand{\name}[1]{#1}", &vec![TexCommand::newcommand(r"\name", "#1").with_args(1)]);
    }

    #[test]
    fn basic_optional_arg() {
        test_valid(r"\newcommand{\area}[2][m^2]{#1 \times #2}", 
            &vec![TexCommand::newcommand(r"\area", r"#1\times #2")
            .with_defaults(2, r"m^2")]);
    }

    #[test]
    fn nested_commands() {
        test_valid(r"\newcommand{\mycommand}[1]{\textbf{#1}}", 
            &vec![TexCommand::newcommand(r"\mycommand", r"\textbf{#1}").with_args(1)]);
    }

    #[test]
    fn renewcommand() {
        test_valid(r"\renewcommand{\emph}[1]{\underline{#1}}", 
            &vec![TexCommand::renewcommand(r"\emph", r"\underline{#1}").with_args(1)]);
    }

    #[test]
    fn mathoperator_basic() {
        test_valid(r"\DeclareNewMathOperator{\myOperator}{sin}", 
            &vec![TexCommand::declare_math_operator(r"\myOperator", r"sin")]);
    }

    #[test]
    fn missing_argument_count() {
        test_invalid(r"\newcommand{\myOperator}[default]{sin}");
    }

    #[test]
    fn wrong_command() {
        test_invalid(r"\newcommans{\myOperator}{sin}");
    }

    #[test]
    fn wrong_command2() {
        test_invalid(r"\newcommando{\myOperator}{sin}");
    }

    #[test]
    fn missing_body() {
        test_invalid(r"\newcommand{\myOperator}");
    }
}
