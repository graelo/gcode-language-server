//! GCode Parser
//!
//! Clean, fast parsing of GCode with minimal allocations.
//! Focused solely on tokenization and AST construction.

pub mod ast;
pub mod lexer;

pub use ast::{Command, Comment, Parameter, ParsedLine};
pub use lexer::{tokenize_line, Token, TokenKind};

/// Parse a single line of GCode into structured data
///
/// This is the main entry point for parsing. It tokenizes the line
/// and constructs a simple AST representation.
pub fn parse_line(line: &str) -> ParsedLine {
    let tokens = lexer::tokenize_line(line);
    ast::tokens_to_parsed_line(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let result = parse_line("G1 X10 Y20");

        if let ParsedLine::Command(cmd) = result {
            assert_eq!(cmd.name, "G1");
            assert_eq!(cmd.parameters.len(), 2);
            assert_eq!(cmd.parameters[0].letter, 'X');
            assert_eq!(cmd.parameters[0].value, "10");
        } else {
            panic!("Expected command");
        }
    }

    #[test]
    fn test_parse_with_comment() {
        let result = parse_line("G1 X10 ; move to X10");

        if let ParsedLine::Command(cmd) = result {
            assert_eq!(cmd.name, "G1");
            assert_eq!(
                cmd.comment,
                Some(Comment {
                    text: " move to X10".to_string()
                })
            );
        } else {
            panic!("Expected command");
        }
    }

    #[test]
    fn test_parse_comment_only() {
        let result = parse_line("; this is a comment");

        if let ParsedLine::Comment(comment) = result {
            assert_eq!(comment.text, " this is a comment");
        } else {
            panic!("Expected comment");
        }
    }

    #[test]
    fn test_parse_empty_line() {
        let result = parse_line("   ");
        assert!(matches!(result, ParsedLine::Empty));
    }
}
