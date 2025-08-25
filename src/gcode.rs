//! Minimal streaming, line-oriented G-code tokenizer and parser.
//!
//! Design goals:
//! - Stream lines (no full-file allocation) suitable for large files.
//! - Produce tokens: Command, Param, Comment.
//! - Provide function to find token at a byte position in the whole text.
//! - Support streaming iteration and lightweight AST for diagnostics.

use std::borrow::Cow;

/// Represents a G-code token with its type, text, and byte positions
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: Cow<'a, str>,
    pub start: usize, // byte offset
    pub end: usize,   // byte offset (exclusive)
}

/// The kind of a G-code token
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Command,
    Param,
    Comment,
}

/// Represents a G-code command like "G1 X10 Y20 ; comment"
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub code: String,
    pub params: Vec<Parameter>,
    pub comment: Option<String>,
}

/// Represents a parameter in a G-code command like "X10.5"
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub letter: char,
    pub value: f64,
}

/// Simple and fast tokenize_line function using character iteration
pub fn tokenize_line(line: &str, offset: usize) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();

    while let Some((start_idx, ch)) = chars.next() {
        let absolute_start = offset + start_idx;

        match ch {
            // Skip whitespace
            ' ' | '\t' => continue,

            // Semicolon comment: consume rest of line
            ';' => {
                let text = &line[start_idx..];
                let token = Token {
                    kind: TokenKind::Comment,
                    text: Cow::Borrowed(text),
                    start: absolute_start,
                    end: offset + line.len(),
                };
                tokens.push(token);
                break;
            }

            // Parenthetical comment
            '(' => {
                let mut end_idx = start_idx + 1;
                let mut found_close = false;

                for (idx, ch) in chars.by_ref() {
                    end_idx = idx;
                    if ch == ')' {
                        end_idx = idx + 1;
                        found_close = true;
                        break;
                    }
                }

                if !found_close {
                    end_idx = line.len();
                }

                let text = &line[start_idx..end_idx];
                let token = Token {
                    kind: TokenKind::Comment,
                    text: Cow::Borrowed(text),
                    start: absolute_start,
                    end: offset + end_idx,
                };
                tokens.push(token);
            }

            // Command or parameter: letter followed by optional digits/dots
            c if c.is_ascii_alphabetic() => {
                let mut end_idx = start_idx + 1;

                // Consume alphanumeric characters, dots, and minus signs
                while let Some(&(idx, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_alphanumeric() || next_ch == '.' || next_ch == '-' {
                        end_idx = idx + 1;
                        chars.next();
                    } else {
                        break;
                    }
                }

                let text = &line[start_idx..end_idx];

                // Determine if it's a command or parameter
                let kind = if is_command_text(text) {
                    TokenKind::Command
                } else {
                    TokenKind::Param
                };

                let token = Token {
                    kind,
                    text: Cow::Borrowed(text),
                    start: absolute_start,
                    end: offset + end_idx,
                };
                tokens.push(token);
            }

            // Skip other characters
            _ => continue,
        }
    }

    tokens
}

fn is_command_text(text: &str) -> bool {
    // Simpler heuristic: commands are tokens that start with G, M or T (case-insensitive).
    // Other single-letter + number tokens (X/Y/Z/E/S/F...) are treated as params.
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => matches!(c.to_ascii_uppercase(), 'G' | 'M' | 'T'),
        None => false,
    }
}

/// Convenience function to tokenize a complete text into tokens.
/// For large files, prefer streaming with TokenIterator.
pub fn tokenize_text(text: &str) -> Vec<Token<'_>> {
    let mut all_tokens = Vec::new();
    let mut current_offset = 0;

    for line in text.lines() {
        let line_tokens = tokenize_line(line, current_offset);
        all_tokens.extend(line_tokens);
        current_offset += line.len() + 1; // +1 for the newline character
    }

    all_tokens
}

/// Find the token at the given byte position
pub fn token_at_position<'a>(tokens: &'a [Token<'a>], position: usize) -> Option<&'a Token<'a>> {
    tokens
        .iter()
        .find(|token| position >= token.start && position < token.end)
}

/// Streaming iterator for reading G-code tokens from a BufRead source
pub struct TokenIterator<R: std::io::BufRead> {
    reader: R,
    current_offset: usize,
    line_buffer: String,
    current_tokens: Vec<Token<'static>>,
    token_index: usize,
}

impl<R: std::io::BufRead> TokenIterator<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            current_offset: 0,
            line_buffer: String::new(),
            current_tokens: Vec::new(),
            token_index: 0,
        }
    }
}

impl<R: std::io::BufRead> Iterator for TokenIterator<R> {
    type Item = Token<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Return current token if available
            if self.token_index < self.current_tokens.len() {
                let token = self.current_tokens[self.token_index].clone();
                self.token_index += 1;
                return Some(token);
            }

            // Read next line
            self.line_buffer.clear();
            match self.reader.read_line(&mut self.line_buffer) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    // Remove trailing newline for tokenization
                    let line = self
                        .line_buffer
                        .trim_end_matches('\n')
                        .trim_end_matches('\r');

                    // Tokenize line and convert to owned tokens
                    let line_tokens = tokenize_line(line, self.current_offset);
                    self.current_tokens = line_tokens
                        .into_iter()
                        .map(|t| Token {
                            kind: t.kind,
                            text: Cow::Owned(t.text.into_owned()),
                            start: t.start,
                            end: t.end,
                        })
                        .collect();

                    self.token_index = 0;
                    self.current_offset += self.line_buffer.len();
                }
                Err(_) => return None,
            }
        }
    }
}

/// Parse tokens into a Command structure for AST operations
pub fn parse_command_from_tokens(tokens: &[Token]) -> Option<Command> {
    if tokens.is_empty() {
        return None;
    }

    let mut command_code = None;
    let mut params = Vec::new();
    let mut comment = None;

    for token in tokens {
        match token.kind {
            TokenKind::Command => {
                command_code = Some(token.text.to_string());
            }
            TokenKind::Param => {
                if let Some(param) = parse_parameter_simple(&token.text) {
                    params.push(param);
                }
            }
            TokenKind::Comment => {
                comment = Some(token.text.to_string());
            }
        }
    }

    command_code.map(|code| Command {
        code,
        params,
        comment,
    })
}

/// Parse a parameter string like "X10.5" into a Parameter struct using nom
fn parse_parameter_simple(text: &str) -> Option<Parameter> {
    // Simple manual parsing - faster than nom for this simple case
    if text.len() < 2 {
        return None;
    }

    let mut chars = text.chars();
    let letter = chars.next()?;
    if !letter.is_ascii_alphabetic() {
        return None;
    }

    let value_str: String = chars.collect();
    let value = value_str.parse::<f64>().ok()?;

    Some(Parameter { letter, value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_line_basic() {
        let tokens = tokenize_line("G1 X10.0 Y5.2 ; move command", 0);
        assert_eq!(tokens.len(), 4);

        assert_eq!(tokens[0].kind, TokenKind::Command);
        assert_eq!(tokens[0].text, "G1");
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 2);

        assert_eq!(tokens[1].kind, TokenKind::Param);
        assert_eq!(tokens[1].text, "X10.0");

        assert_eq!(tokens[2].kind, TokenKind::Param);
        assert_eq!(tokens[2].text, "Y5.2");

        assert_eq!(tokens[3].kind, TokenKind::Comment);
        assert_eq!(tokens[3].text, "; move command");
    }

    #[test]
    fn token_at_position_basic() {
        let tokens = tokenize_line("G1 X10.0 Y5.2", 0);
        let token = token_at_position(&tokens, 3).unwrap(); // Should be X10.0
        assert_eq!(token.text, "X10.0");
        assert_eq!(token.kind, TokenKind::Param);

        let token = token_at_position(&tokens, 0).unwrap(); // Should be G1
        assert_eq!(token.text, "G1");
        assert_eq!(token.kind, TokenKind::Command);
    }

    #[test]
    fn parse_command_from_tokens_basic() {
        let line = "G1 X10.0 Y5.2 ; move command";
        let tokens = tokenize_line(line, 0);
        let command = parse_command_from_tokens(&tokens).expect("command");

        assert_eq!(command.code, "G1");
        assert_eq!(command.params.len(), 2);
        assert_eq!(command.params[0].letter, 'X');
        assert_eq!(command.params[0].value, 10.0);
        assert_eq!(command.params[1].letter, 'Y');
        assert_eq!(command.params[1].value, 5.2);
        assert!(command.comment.is_some());
        assert!(command.comment.unwrap().contains("move command"));
    }

    #[test]
    fn parse_parameter_basic() {
        assert_eq!(
            parse_parameter_simple("X10.5").unwrap(),
            Parameter {
                letter: 'X',
                value: 10.5
            }
        );
        assert_eq!(
            parse_parameter_simple("S200").unwrap(),
            Parameter {
                letter: 'S',
                value: 200.0
            }
        );
        assert!(parse_parameter_simple("").is_none());
        assert!(parse_parameter_simple("10.5").is_none()); // no letter
    }

    #[test]
    fn streaming_iterator_basic() {
        use std::io::Cursor;
        let content = "G28 ; home\nM104 S200\nG1 X0 Y0\n";
        let cursor = Cursor::new(content.as_bytes());
        let tokens: Vec<_> = TokenIterator::new(cursor).collect();

        // Should have tokens from all lines
        let commands: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Command)
            .collect();
        assert!(commands.len() >= 3); // G28, M104, G1
    }

    #[test]
    fn missing_newline_at_eof() {
        // last line has no trailing newline
        let text = "G1 X1 Y1\nG0 X0 Y0"; // no '\n' at EOF
        let tokens = tokenize_text(text);
        // there should be a token for the last line's command (G0)
        assert!(tokens.iter().any(|t| t.text == "G0"));
    }

    #[test]
    fn multiple_commands_on_one_line() {
        // a single line containing two commands separated by whitespace
        let text = "G28 G1 X0 Y0\n";
        let tokens = tokenize_text(text);
        // Expect two command tokens at least: G28 and G1
        let commands: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Command)
            .collect();
        assert!(commands.iter().any(|t| t.text == "G28"));
        assert!(commands.iter().any(|t| t.text == "G1"));
    }

    #[test]
    fn long_comment_block() {
        // long comment (simulate large comments) should be captured as a comment token
        let long_comment = "a".repeat(10_000);
        let text = format!("G1 X0 Y0 ;{}\n", long_comment);
        let tokens = tokenize_text(&text);
        let comment = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Comment)
            .expect("comment token");
        assert!(comment.text.len() >= 10_000);
    }

    #[test]
    fn integration_tokenize_sample() {
        let sample = ["G28 ; home", "M104 S200 (set hotend)", "G1 X10 Y10 F1500"];

        // Join the lines with newlines to create a complete text
        let text = sample.join("\n");
        let tokens = tokenize_text(&text);

        // quick sanity: we should have at least one command per line
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Command));

        // find token at a few positions
        let t0 = token_at_position(&tokens, 0).expect("token at 0");
        assert_eq!(t0.kind, TokenKind::Command);

        // position inside comment of first line
        let comment_pos = sample[0].find(';').unwrap() + 1; // 1-based into line
        let global_pos = comment_pos;
        let t_comment = token_at_position(&tokens, global_pos).expect("comment token");
        assert_eq!(t_comment.kind, TokenKind::Comment);
    }
}
