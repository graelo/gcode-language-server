//! GCode Lexer
//!
//! Fast, simple tokenization of GCode lines.
//! Focus: extract tokens quickly with minimal allocations.

/// Token types in GCode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    /// Command like "G1", "M104"
    Command,
    /// Parameter like "X10", "S255"  
    Parameter,
    /// Comment (semicolon or parenthetical)
    Comment,
}

/// A token with its text content
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

/// Tokenize a line of GCode into tokens
///
/// This is much simpler than the current approach - no position tracking,
/// no streaming, just fast extraction of tokens from a line.
pub fn tokenize_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();

    while let Some((start_idx, ch)) = chars.next() {
        match ch {
            // Skip whitespace
            ' ' | '\t' | '\r' | '\n' => continue,

            // Semicolon comment: consume rest of line
            ';' => {
                let text = line[start_idx..].to_string();
                tokens.push(Token {
                    kind: TokenKind::Comment,
                    text,
                });
                break; // Rest of line is comment
            }

            // Parenthetical comment
            '(' => {
                let mut end_idx = start_idx + 1;
                let mut found_close = false;

                for (idx, ch) in chars.by_ref() {
                    if ch == ')' {
                        end_idx = idx + 1;
                        found_close = true;
                        break;
                    }
                    end_idx = idx + 1;
                }

                if !found_close {
                    end_idx = line.len();
                }

                let text = line[start_idx..end_idx].to_string();
                tokens.push(Token {
                    kind: TokenKind::Comment,
                    text,
                });
            }

            // Letter starts command or parameter
            c if c.is_ascii_alphabetic() => {
                let mut end_idx = start_idx + 1;

                // Consume alphanumeric, dots, minus, plus
                while let Some(&(idx, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_alphanumeric()
                        || next_ch == '.'
                        || next_ch == '-'
                        || next_ch == '+'
                    {
                        end_idx = idx + 1;
                        chars.next();
                    } else {
                        break;
                    }
                }

                let text = line[start_idx..end_idx].to_string();

                // Simple heuristic: Commands start with G, M, T
                let kind = if is_command(&text) {
                    TokenKind::Command
                } else {
                    TokenKind::Parameter
                };

                tokens.push(Token { kind, text });
            }

            // Skip other characters (malformed input)
            _ => continue,
        }
    }

    tokens
}

/// Determine if a token is a command
///
/// Simple heuristic: G/M/T codes are commands, everything else is parameter.
/// This works for 99% of GCode and is much simpler than complex pattern matching.
fn is_command(text: &str) -> bool {
    if let Some(first_char) = text.chars().next() {
        matches!(first_char.to_ascii_uppercase(), 'G' | 'M' | 'T')
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_command() {
        let tokens = tokenize_line("G1 X10 Y20");

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Command);
        assert_eq!(tokens[0].text, "G1");
        assert_eq!(tokens[1].kind, TokenKind::Parameter);
        assert_eq!(tokens[1].text, "X10");
        assert_eq!(tokens[2].kind, TokenKind::Parameter);
        assert_eq!(tokens[2].text, "Y20");
    }

    #[test]
    fn test_tokenize_with_semicolon_comment() {
        let tokens = tokenize_line("G1 X10 ; move to X10");

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[2].kind, TokenKind::Comment);
        assert_eq!(tokens[2].text, "; move to X10");
    }

    #[test]
    fn test_tokenize_paren_comment() {
        let tokens = tokenize_line("G1 (rapid move) X10");

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].kind, TokenKind::Comment);
        assert_eq!(tokens[1].text, "(rapid move)");
    }

    #[test]
    fn test_tokenize_comment_only() {
        let tokens = tokenize_line("; this is a comment");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
        assert_eq!(tokens[0].text, "; this is a comment");
    }

    #[test]
    fn test_tokenize_empty_line() {
        let tokens = tokenize_line("   ");
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_is_command() {
        assert!(is_command("G1"));
        assert!(is_command("M104"));
        assert!(is_command("T0"));
        assert!(!is_command("X10"));
        assert!(!is_command("S255"));
    }

    #[test]
    fn test_float_parameters() {
        let tokens = tokenize_line("G1 X10.5 Y-2.3 Z+1.0");

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[1].text, "X10.5");
        assert_eq!(tokens[2].text, "Y-2.3");
        assert_eq!(tokens[3].text, "Z+1.0");
    }
}
