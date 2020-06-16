use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    OpenParen,
    CloseParen,
    Number(&'a str, i64),
    Ident(&'a str, String),
}

pub struct Lexer<'a> {
    input: Chars<'a>,
    // TODO: This is garbage
    mark: &'a str,
    current: char,
    error: Option<String>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut me = Self {
            input: source.chars(),
            mark: source,
            current: '\0',
            error: None,
        };
        me.nompf();
        me.whitespace();
        me
    }

    pub fn is_end(&self) -> bool {
        self.input.as_str().is_empty()
    }

    pub fn mark(&self) -> &'a str {
        self.mark
    }

    pub fn span(&self, start: &'a str) -> &'a str {
        let end = self.mark();
        let length = end.as_ptr() as usize - start.as_ptr() as usize;
        &start[..length]
    }

    fn nompf(&mut self) {
        self.mark = self.input.as_str();
        self.current = self.input.next().unwrap_or('\0');
    }

    fn whitespace(&mut self) {
        while self.current.is_whitespace() {
            self.nompf();
        }
    }

    fn basic(&mut self, tok: Token<'a>) -> Option<Token<'a>> {
        self.nompf();
        self.whitespace();
        Some(tok)
    }

    fn num(&mut self) -> Token<'a> {
        let start = self.mark();
        while self.current.is_ascii_digit() {
            self.nompf();
        }
        let span = self.span(start);
        // TODO: bigint
        let value = span
            .parse()
            .expect("Failed to parse thing we parsed as number");
        self.whitespace();
        // TODO: decimals
        Token::Number(span, value)
    }

    fn ident(&mut self) -> Token<'a> {
        let start = self.mark();
        while self.current.is_ascii_alphabetic() {
            self.nompf();
        }
        let span = self.span(start);
        self.whitespace();
        Token::Ident(span, span.to_string())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            '+' => self.basic(Token::Add),
            '-' => self.basic(Token::Sub),
            '*' => self.basic(Token::Mul),
            '/' => self.basic(Token::Div),
            '%' => self.basic(Token::Mod),
            '^' => self.basic(Token::Pow),
            '(' => self.basic(Token::OpenParen),
            ')' => self.basic(Token::CloseParen),
            '0'..='9' => Some(self.num()),
            'a'..='z' | 'A'..='Z' => Some(self.ident()),
            _ => {
                self.error = Some(format!("Unexpected character {}", self.current));
                None
            }
        }
    }
}
