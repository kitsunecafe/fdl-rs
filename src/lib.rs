/*
    <syntax>      ::= <section> | <section> <syntax>
    <section>     ::= <header> <fields> <endsection>
    <endsection> ::= "[/]" <EOL>
    <header>      ::= "[" <text> "]" <EOL>
    <fields>      ::= <field> | <field> <fields>
    <field>       ::= <text> <whitespace> "=" <whitespace> <text> <EOL>
    <text>        ::= <character> | <character> <text>
    <lineend>     ::= <whitespace> <EOL> | <lineend> <lineend>
    <whitespace>  ::= " " | " " <whitespace>
    <character>   ::= <digit> | <letter> | <symbol>
    <letter>      ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" | "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
    <digit>       ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
    <symbol>      ::=  "|" | " " | "!" | "#" | "$" | "%" | "&" | "(" | ")" | "*" | "+" | "," | "-" | "." | "/" | ":" | ";" | ">" | "=" | "<" | "?" | "@" | "[" | "\" | "]" | "^" | "_" | "`" | "{" | "}" | "~"
    <EOL>         ::= "\n" | "\r\n"
*/

use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    path::Path,
};

const NEWLINE: u8 = '\n' as u8;
const SQ_BR_O: u8 = '[' as u8;
const SQ_BR_C: u8 = ']' as u8;
const SLASH: u8 = '/' as u8;
const EQ: u8 = '=' as u8;

#[derive(Debug, PartialEq)]
pub enum Token {
    SectionStart(String),
    SectionEnd,
    Field(String),
    Value(String),
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(usize),
    EndOfLine,
    EndOfFile,
}

pub struct Reader<T>
where
    T: Seek + Read,
{
    cursor: usize,
    reader: BufReader<T>,
}

impl<T> Reader<T>
where
    T: Seek + Read,
{
    pub fn new(reader: BufReader<T>) -> Self {
        Self { cursor: 0, reader }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn peek(&mut self) -> Option<u8> {
        self.peek_nth(1)
    }

    pub fn peek_nth(&mut self, offset: i64) -> Option<u8> {
        let mut result = None;

        if self.reader.seek_relative(offset).is_ok() {
            if let Ok(buf) = self.reader.fill_buf() {
                if buf.len() > 0 {
                    result = Some(buf[0]);
                }
            }

            self.reader.seek_relative(-offset).expect("");
        }

        result
    }

    pub fn consume(&mut self) -> Option<u8> {
        self.consume_nth(1)
    }

    pub fn consume_if(&mut self, target: u8) -> bool {
        match self.peek() {
            Some(character) if character == target => {
                self.consume();
                true
            }
            Some(_) | None => false,
        }
    }

    pub fn consume_until(&mut self, target: u8) -> Vec<u8> {
        let mut seq = Vec::new();

        while let Some(ch) = self.peek() {
            if ch != target {
                self.consume();
                seq.push(ch);
            } else {
                break;
            }
        }

        seq
    }

    pub fn consume_until_newline_or(&mut self, target: u8) -> Result<Vec<u8>, LexerError> {
        let mut seq = Vec::new();

        while let Some(ch) = self.peek() {
            if ch == NEWLINE {
                return Err(LexerError::EndOfLine);
            }

            if ch != target {
                self.consume();
                seq.push(ch);
            } else {
                return Ok(seq);
            }
        }

        Err(LexerError::EndOfFile)
    }

    pub fn consume_nth(&mut self, offset: i64) -> Option<u8> {
        if self.reader.seek_relative(offset).is_ok() {
            self.cursor += offset as usize;
            if let Ok(buf) = self.reader.fill_buf() {
                return Some(buf[0]);
            }
        }

        None
    }

    pub fn is_eof(&mut self) -> bool {
        self.peek().is_none()
    }
}

pub struct Lexer<T>
where
    T: Seek + Read,
{
    reader: Reader<T>,
}

impl<T> Lexer<T>
where
    T: Seek + Read,
{
    pub fn new(reader: Reader<T>) -> Self {
        Self { reader }
    }

    fn section(&mut self) -> Option<Token> {
        if let Some(char) = self.reader.peek() {
            if char == SQ_BR_O {
                self.reader.consume();
                if let Ok(seq) = self.reader.consume_until_newline_or(SQ_BR_C) {
                    self.reader.consume();
                    if seq.len() == 1 && seq[0] == SLASH {
                        return Some(Token::SectionEnd);
                    } else {
                        return Some(Token::SectionStart(
                            String::from_utf8_lossy(&seq).to_string(),
                        ));
                    }
                }
            }
        }

        None
    }

    fn field(&mut self) -> Option<Token> {
        if let Ok(seq) = self.reader.consume_until_newline_or(EQ) {
            Some(Token::Field(String::from_utf8_lossy(&seq).to_string()))
        } else {
            None
        }
    }

    fn value(&mut self) -> Option<Token> {
        match self.reader.peek() {
            Some(ch) if ch == EQ => {
                self.reader.consume();
                let seq = self.reader.consume_until(NEWLINE);
                Some(Token::Value(String::from_utf8_lossy(&seq).to_string()))
            }
            Some(_) | None => None,
        }
    }

    fn newline(&mut self) -> bool {
        self.reader.consume_if(NEWLINE)
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut result: Vec<Token> = Vec::new();

        loop {
            if self.reader.is_eof() {
                break;
            } else if self.newline() {
                continue;
            } else if let Some(section) = self.section() {
                result.push(section);
            } else if let Some(value) = self.value() {
                result.push(value);
            } else if let Some(field) = self.field() {
                result.push(field);
            } else {
                return Err(LexerError::UnexpectedCharacter(self.reader.cursor()));
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub value: String,
}

pub struct Parser;

impl Parser {
    pub fn parse(tokens: &mut VecDeque<Token>) -> Vec<Section> {
        let mut tree: Vec<Section> = Vec::new();

        while let Some(token) = tokens.pop_front() {
            match token {
                Token::SectionStart(name) => {
                    let fields = Parser::collect_fields(tokens);
                    tree.push(Section { name, fields });
                }
                _ => {
                    continue;
                }
            }
        }

        tree
    }

    fn collect_fields(tokens: &mut VecDeque<Token>) -> Vec<Field> {
        let mut fields: Vec<Field> = Vec::new();

        while let Some(token) = tokens.pop_front() {
            match token {
                Token::SectionEnd => break,
                Token::Field(name) => {
                    if let Some(Token::Value(value)) = tokens.pop_front() {
                        fields.push(Field { name, value });
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        fields
    }
}

#[derive(Debug, Clone)]
pub struct FDL {
    tree: Vec<Section>,
}

impl FDL {
    pub fn load_from_file<P>(filename: P) -> Result<Self, &'static str>
    where
        P: AsRef<Path>,
    {
        if let Ok(handle) = File::open(filename) {
            let reader = Reader::new(BufReader::new(handle));
            let mut lexer = Lexer::new(reader);

            if let Ok(tokens) = lexer.lex() {
                Ok(Self {
                    tree: Parser::parse(&mut VecDeque::from(tokens)),
                })
            } else {
                Err("could not parse file")
            }
        } else {
            Err("could not open file")
        }
    }

    pub fn fetch(&self, section: &str, field: &str) -> Option<&str> {
        if let Some(section) = self.tree.iter().find(|s| s.name == section) {
            if let Some(field) = section.fields.iter().find(|f| f.name == field) {
                Some(field.value.as_str())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FDL;

    #[test]
    fn load_file() {
        if let Ok(fdl) = FDL::load_from_file("test.fdl") {
            if let Some(flap_frames) = fdl.fetch("flap", "frames") {
                assert_eq!(flap_frames, "1");
            } else {
                panic!("could not fetch value");
            }
        } else {
            panic!("could not open file");
        }
    }
}
