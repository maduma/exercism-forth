use std::collections::HashMap;

pub type Value = i32;
pub type Result = std::result::Result<(), Error>;

#[derive(Debug, Clone)]
pub struct Forth {
    stack: Vec<Value>,
    expanded_definitions: HashMap<String, Operation>,
    raw_definitions: Vec<(String, Vec<Token>)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}

#[derive(Debug, Clone,PartialEq)]
enum Operation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Duplicate,
    Drop,
    Swap,
    Over,
    UserDefined(Vec<Token>),
}

enum Command {
    Expression(Vec<Token>),
    Definition(String, Vec<Token>),
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Word(String),
    Number(Value),
    NativeOperation(Operation),
    UserDefinedOperation(String, Vec<Token>),
}

const PREDIFINED_OPERATIONS: [(&str, Operation); 8] = [
    ("+", Operation::Addition),
    ("-", Operation::Subtraction),
    ("*", Operation::Multiplication),
    ("/", Operation::Division),
    ("dup", Operation::Duplicate),
    ("drop", Operation::Drop),
    ("swap", Operation::Swap),
    ("over", Operation::Over),
];

fn do_operation(op: &Operation) -> fn(&mut Vec<Value>) -> Result {
    match op {
        Operation::Addition => do_addition,
        Operation::Subtraction => do_substraction,
        Operation::Multiplication => do_multiplication,
        Operation::Division => do_division,
        Operation::Duplicate => do_dup,
        Operation::Drop => do_drop,
        Operation::Swap => do_swap,
        Operation::Over => do_over,
        _ => do_nothing,
    }
}

fn do_addition(stack: &mut Vec<Value>) -> Result {
    let a = stack.pop().ok_or(Error::StackUnderflow)?;
    let b = stack.pop().ok_or(Error::StackUnderflow)?;
    stack.push(a + b);
    Ok(())
}

fn do_substraction(stack: &mut Vec<Value>) -> Result {
    let a = stack.pop().ok_or(Error::StackUnderflow)?;
    let b = stack.pop().ok_or(Error::StackUnderflow)?;
    stack.push(b - a);
    Ok(())
}

fn do_multiplication(stack: &mut Vec<Value>) -> Result {
    let a = stack.pop().ok_or(Error::StackUnderflow)?;
    let b = stack.pop().ok_or(Error::StackUnderflow)?;
    stack.push(a * b);
    Ok(())
}

fn do_division(stack: &mut Vec<Value>) -> Result {
    let a = stack.pop().ok_or(Error::StackUnderflow)?;
    if a == 0 {
        return Err(Error::DivisionByZero);
    }
    let b = stack.pop().ok_or(Error::StackUnderflow)?;
    stack.push(b / a);
    Ok(())
}

fn do_dup(stack: &mut Vec<Value>) -> Result {
    let a = stack.pop().ok_or(Error::StackUnderflow)?;
    stack.push(a);
    stack.push(a);
    Ok(())
}

fn do_drop(stack: &mut Vec<Value>) -> Result {
    stack.pop().ok_or(Error::StackUnderflow)?;
    Ok(())
}

fn do_swap(stack: &mut Vec<Value>) -> Result {
    let a = stack.pop().ok_or(Error::StackUnderflow)?;
    let b = stack.pop().ok_or(Error::StackUnderflow)?;
    stack.push(a);
    stack.push(b);
    Ok(())
}

fn do_over(stack: &mut Vec<Value>) -> Result {
    let a = stack.pop().ok_or(Error::StackUnderflow)?;
    let b = stack.pop().ok_or(Error::StackUnderflow)?;
    stack.push(b);
    stack.push(a);
    stack.push(b);
    Ok(())
}

#[allow(clippy::ptr_arg)]
fn do_nothing(_stack: &mut Vec<Value>) -> Result {
    Ok(())
}

fn parse_tokens(input: &str) -> Vec<Token> {
    input
        .to_lowercase()
        .split_whitespace()
        .map(|s| match s.parse::<i32>() {
            Ok(i) => Token::Number(i),
            _ => Token::Word(s.to_string()),
        })
        .collect()
}

fn is_definition(tokens: &[Token]) -> std::result::Result<bool, Error> {
    let empty = &Token::Word("".to_string());
    let colon = &Token::Word(":".to_string());
    let semicolon = &Token::Word(";".to_string());
    let fst = tokens.first().unwrap_or(empty);
    let lst = tokens.last().unwrap_or(empty);
    if fst == colon {
        if lst == semicolon {
            Ok(true)
        } else {
            Err(Error::InvalidWord)
        }
    } else {
        Ok(false)
    }
}

fn parse_command(input: &str) -> std::result::Result<Command, Error> {
    let tokens = parse_tokens(input);
    if is_definition(&tokens)? {
        if let Token::Word(str) = &tokens[1] {
            let tokens = tokens[2..tokens.len() - 1].to_vec();
            Ok(Command::Definition(str.to_owned(), tokens))
        } else {
            return Err(Error::InvalidWord);
        }
    } else {
        Ok(Command::Expression(tokens))
    }
}

impl Default for Forth {
    fn default() -> Self {
        let predifined = PREDIFINED_OPERATIONS
            .into_iter()
            .map(|(s, o)| (s.to_string(), o))
            .collect();
        Forth {
            stack: Vec::new(),
            expanded_definitions: predifined,
            raw_definitions: Vec::new(),
        }
    }
}

fn split_commands(input: &str) -> Vec<String> {
    let tmp = input.chars().fold(
        (Vec::<String>::new(), String::new()),
        |mut acc, c| match c {
            ':' => {
                acc.0.push(acc.1);
                (acc.0, c.to_string())
            }
            ';' => {
                let mut s = acc.1;
                s.push(c);
                acc.0.push(s);
                (acc.0, String::new())
            }
            _ => {
                let mut s = acc.1;
                s.push(c);
                (acc.0, s)
            }
        },
    );
    let mut cmds = tmp.0;
    if !tmp.1.is_empty() {
        cmds.push(tmp.1)
    };
    cmds.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn append_front(tokens: &mut Vec<Token>, mut op_tokens: Vec<Token>) {
    op_tokens.reverse();
    for t in op_tokens {
        tokens.insert(0, t)
    }
}

impl Forth {
    pub fn new() -> Forth {
        Forth::default()
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    fn lookup_word(&mut self, input: &str) -> std::result::Result<Operation, Error> {
        self.expanded_definitions
            .get(input)
            .cloned()
            .ok_or(Error::UnknownWord)
    }

    fn is_raw_definition(&self, input: &str) -> bool {
        self.raw_definitions.iter().any(|(name, _)| name == input)
    }

    fn expand_word(&mut self, word: &str) -> std::result::Result<Operation, Error> {
        if !self.expanded_definitions.contains_key(word) || self.is_raw_definition(word) {
            while !self.raw_definitions.is_empty() {
                let (name, tokens) = self.raw_definitions.remove(0);
                let tokens = self.expand_raw_definition(tokens);
                self.expanded_definitions
                    .insert(name, Operation::UserDefined(tokens));
            }
        }
        self.lookup_word(word)
    }

    fn expand_raw_definition(&mut self, mut tokens: Vec<Token>) -> Vec<Token> {
        let mut buf = Vec::new();
        while !tokens.is_empty() {
            let token = tokens.remove(0);
            match token {
                Token::Number(_) => buf.push(token),
                Token::Word(input) => {
                    if let Ok(op) = self.lookup_word(&input) {
                        match op {
                            Operation::UserDefined(_tokens) => append_front(&mut tokens, _tokens),
                            _ => buf.push(Token::Word(input)),
                        }
                    }
                },
                _ => (),
            }
        }
        buf
    }

    pub fn eval(&mut self, input: &str) -> Result {
        for command in split_commands(input) {
            self.eval_command(&command)?
        }
        Ok(())
    }

    fn eval_command(&mut self, command: &str) -> Result {
        match parse_command(command)? {
            Command::Definition(name, tokens) => self.raw_definitions.push((name, tokens)),
            Command::Expression(mut tokens) => {
                while !tokens.is_empty() {
                    let token = tokens.remove(0);
                    match token {
                        Token::Number(i) => self.stack.push(i),
                        Token::Word(str) => match self.expand_word(&str)? {
                            Operation::UserDefined(op_tokens) => {
                                append_front(&mut tokens, op_tokens)
                            }
                            op @ _ => do_operation(&op)(&mut self.stack)?,
                        },
                        Token::NativeOperation(op) => do_operation(&op)(&mut self.stack)?,
                        Token::UserDefinedOperation(_name, op_tokens) => append_front(&mut tokens, op_tokens),
                    }
                }
            }
        }
        Ok(())
    }
}
