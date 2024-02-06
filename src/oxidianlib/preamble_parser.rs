
#[derive(Debug)]
struct Lexer<T: Iterator<Item=char>> {
    buffer: String,
    text: T,
    state: LexerState
}


#[derive(Debug)]
enum LexerState {
    Normal, 
    Comment, 
    Full,
}


#[derive(Debug, PartialEq)]
enum Token {
    Text(String),
    Newcommand,
    DeclareMathOperator(bool),
    Comment,
    Renewcommand, 
    CommandName(String),
    OpenCurly,
    CloseCurly, 
    OpenBracket,
    CloseBracket
}


impl<T: Iterator<Item=char>> Lexer<T> {

    pub fn new(text: T) -> Self {
        Self { buffer: String::new(), text, state: LexerState::Normal }
    }

    fn flush<'a>(&'a mut self) -> Option<Token> {
        let result = match self.buffer.as_str().trim() {
            r"\newcommand" => { Some(Token::Newcommand) },
            r"\DeclareMathOperator" => { Some(Token::DeclareMathOperator(false)) },
            r"\DeclareMathOperator*" => { Some(Token::DeclareMathOperator(true)) },
            r"\renewcommand" => { Some(Token::Renewcommand) },
            r"{" => { Some(Token::OpenCurly) },
            r"}" => { Some(Token::CloseCurly) },
            r"[" => { Some(Token::OpenBracket) },
            r"]" => { Some(Token::CloseBracket) }, 
            r""  => { None },
            other_text => {
                if other_text.starts_with(r"\") {
                    Some(Token::CommandName(other_text.to_string()))
                } else if other_text.starts_with(r"%") {
                    Some(Token::Comment)
                }
                else {
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
                },
                ']' | '[' | '{' | '}' => {
                    self.state = LexerState::Full;
                    self.flush() 
                },
                '\\' | '\n' => {
                    self.state = LexerState::Normal;
                    self.flush()
                }, 
                _   => { 
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
            LexerState::Normal => { self.parse_char_normal(new_char) },
            LexerState::Comment => { self.parse_char_comment(new_char) },
            LexerState::Full => { self.parse_char_full(new_char) }
        }
    }
}


impl<T: Iterator<Item=char>> Iterator for Lexer<T> {

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


#[cfg(test)]
mod tests {
    use super::*;

    fn test_correctness<U, T>(input: U, expected_tokens: T) 
    where 
        U: AsRef<str>,
        T: Iterator<Item=Token>
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
            Token::CloseCurly
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
            Token::Comment
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
            Token::Comment
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
            Token::Comment
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
            Token::Comment,
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
            Token::OpenCurly, Token::Comment,
            Token::CommandName(r"\test".to_string()),
            Token::Comment,
            Token::CloseCurly,
            Token::OpenCurly, Token::Comment,
            Token::Text("argument".to_string()), Token::Comment,
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
            Token::OpenCurly, Token::Comment,
            Token::CommandName(r"\test".to_string()),
            Token::Comment,
            Token::CloseCurly,
            Token::OpenCurly, Token::Comment,
            Token::Text("argument".to_string()), Token::Comment,
            Token::CloseCurly,
        ];
        test_correctness(input_string, parsed.into_iter());
    }
}
