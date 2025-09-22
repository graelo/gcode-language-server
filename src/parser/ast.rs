//! Abstract Syntax Tree for GCode
//!
//! Clean, minimal types representing parsed GCode structure.
//! No validation logic or LSP concerns - pure data representation.

use crate::parser::lexer::{Token, TokenKind};

/// A parsed line of GCode
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedLine {
    /// A GCode command with parameters and optional comment
    Command(Command),
    /// A comment-only line  
    Comment(Comment),
    /// An empty or whitespace-only line
    Empty,
}

/// A GCode command like "G1" or "M104"
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// Command name (e.g., "G1", "M104")
    pub name: String,
    /// Command parameters (e.g., X10, Y20)
    pub parameters: Vec<Parameter>,
    /// Optional trailing comment
    pub comment: Option<Comment>,
}

/// A command parameter like "X10" or "S255"
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Parameter letter (e.g., 'X', 'Y', 'S')
    pub letter: char,
    /// Parameter value as string (parsing to numbers happens in validation)
    pub value: String,
}

/// A comment (semicolon or parenthetical)
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// Comment text (without the delimiters)
    pub text: String,
}

/// Convert tokens into a parsed line
///
/// This is where the simple parsing logic lives - much cleaner than
/// the current mixed tokenization/parsing/validation approach.
pub fn tokens_to_parsed_line(tokens: Vec<Token>) -> ParsedLine {
    if tokens.is_empty() {
        return ParsedLine::Empty;
    }

    // Find command token
    let command_token = tokens.iter().find(|t| t.kind == TokenKind::Command);

    if let Some(cmd_token) = command_token {
        // Extract parameters
        let parameters: Vec<Parameter> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Parameter)
            .filter_map(|t| parse_parameter_token(&t.text))
            .collect();

        // Extract comment
        let comment = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Comment)
            .map(|t| Comment {
                text: extract_comment_text(&t.text),
            });

        ParsedLine::Command(Command {
            name: cmd_token.text.clone(),
            parameters,
            comment,
        })
    } else {
        // Check if it's a comment-only line
        if let Some(comment_token) = tokens.iter().find(|t| t.kind == TokenKind::Comment) {
            ParsedLine::Comment(Comment {
                text: extract_comment_text(&comment_token.text),
            })
        } else {
            ParsedLine::Empty
        }
    }
}

/// Parse a parameter token like "X10.5" into a Parameter
fn parse_parameter_token(text: &str) -> Option<Parameter> {
    if text.len() < 2 {
        return None;
    }

    let mut chars = text.chars();
    let letter = chars.next()?;

    if !letter.is_ascii_alphabetic() {
        return None;
    }

    let value = chars.collect::<String>();

    Some(Parameter { letter, value })
}

/// Extract comment text, removing delimiters
fn extract_comment_text(text: &str) -> String {
    if let Some(stripped) = text.strip_prefix(';') {
        stripped.to_string()
    } else if text.starts_with('(') && text.ends_with(')') {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::{Token, TokenKind};

    #[test]
    fn test_parse_parameter_token() {
        let param = parse_parameter_token("X10.5").unwrap();
        assert_eq!(param.letter, 'X');
        assert_eq!(param.value, "10.5");
    }

    #[test]
    fn test_extract_semicolon_comment() {
        let text = extract_comment_text("; this is a comment");
        assert_eq!(text, " this is a comment");
    }

    #[test]
    fn test_extract_paren_comment() {
        let text = extract_comment_text("(this is a comment)");
        assert_eq!(text, "this is a comment");
    }

    #[test]
    fn test_tokens_to_command() {
        let tokens = vec![
            Token {
                kind: TokenKind::Command,
                text: "G1".to_string(),
            },
            Token {
                kind: TokenKind::Parameter,
                text: "X10".to_string(),
            },
            Token {
                kind: TokenKind::Parameter,
                text: "Y20".to_string(),
            },
        ];

        let result = tokens_to_parsed_line(tokens);

        if let ParsedLine::Command(cmd) = result {
            assert_eq!(cmd.name, "G1");
            assert_eq!(cmd.parameters.len(), 2);
            assert_eq!(cmd.parameters[0].letter, 'X');
            assert_eq!(cmd.parameters[0].value, "10");
        } else {
            panic!("Expected command");
        }
    }
}
