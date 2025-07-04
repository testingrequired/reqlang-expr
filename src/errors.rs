//! Errors

use lalrpop_util::ParseError;
use thiserror::Error;

use crate::{
    lexer::Token,
    span::{Span, Spanned},
    types::Type,
};

pub type ExprResult<T> = std::result::Result<T, Vec<ExprErrorS>>;

#[derive(Debug, Error, PartialEq)]
pub enum ExprError {
    #[error("There was an error lexing expression: {0}")]
    LexError(#[from] LexicalError),
    #[error("There was an error in the expression syntax: {0}")]
    SyntaxError(#[from] SyntaxError),
    #[error("There was a compliation error with the expression: {0}")]
    CompileError(#[from] CompileError),
    #[error("There was a runtime error with the expression: {0}")]
    RuntimeError(#[from] RuntimeError),
}

impl diagnostics::AsDiagnostic for ExprError {
    fn as_diagnostic(&self, source: &str, span: &Span) -> diagnostics::ExprDiagnostic {
        match self {
            ExprError::LexError(e) => e.as_diagnostic(source, span),
            ExprError::CompileError(e) => e.as_diagnostic(source, span),
            ExprError::SyntaxError(e) => e.as_diagnostic(source, span),
            ExprError::RuntimeError(e) => e.as_diagnostic(source, span),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Error)]
pub enum LexicalError {
    #[default]
    #[error("Invalid token")]
    InvalidToken,
}

impl diagnostics::AsDiagnostic for LexicalError {
    fn as_diagnostic(&self, source: &str, span: &Span) -> diagnostics::ExprDiagnostic {
        match self {
            LexicalError::InvalidToken => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum SyntaxError {
    #[error("extraneous input: {token:?}")]
    ExtraToken { token: String },
    #[error("invalid input")]
    InvalidToken,
    #[error("unexpected input: {token:?}")]
    UnexpectedInput { token: String },
    #[error("unexpected end of file; expected: {expected:?}")]
    UnrecognizedEOF { expected: Vec<String> },
    #[error("unexpected {token:?}; expected: {expected:?}")]
    UnrecognizedToken {
        token: String,
        expected: Vec<String>,
    },
    #[error("unterminated string")]
    UnterminatedString,
}

impl SyntaxError {
    pub fn from_parser_error(
        err: ParseError<usize, Token, ExprErrorS>,
        source: &str,
    ) -> ExprErrorS {
        match err {
            ParseError::InvalidToken { location } => {
                (SyntaxError::InvalidToken.into(), location..location)
            }
            ParseError::UnrecognizedEof { location, expected } => (
                SyntaxError::UnrecognizedEOF { expected }.into(),
                location..location,
            ),
            ParseError::UnrecognizedToken {
                token: (start, _, end),
                expected,
            } => (
                SyntaxError::UnrecognizedToken {
                    token: source[start..end].to_string(),
                    expected,
                }
                .into(),
                start..end,
            ),
            ParseError::ExtraToken {
                token: (start, _, end),
            } => (
                SyntaxError::ExtraToken {
                    token: source[start..end].to_string(),
                }
                .into(),
                start..end,
            ),
            ParseError::User { error } => error,
        }
    }
}

impl diagnostics::AsDiagnostic for SyntaxError {
    fn as_diagnostic(&self, source: &str, span: &Span) -> diagnostics::ExprDiagnostic {
        match self {
            SyntaxError::ExtraToken { token: _ } => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            SyntaxError::InvalidToken => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            SyntaxError::UnexpectedInput { token: _ } => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            SyntaxError::UnrecognizedEOF { expected: _ } => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            SyntaxError::UnrecognizedToken {
                token: _,
                expected: _,
            } => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            SyntaxError::UnterminatedString => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CompileError {
    #[error("undefined: {0}")]
    Undefined(String),
    #[error("expects {expected} arguments but received {actual}")]
    WrongNumberOfArgs { expected: usize, actual: usize },
    #[error("call expression without a callee")]
    NoCallee,
    #[error("expected type {expected} but received {actual}")]
    TypeMismatch { expected: Type, actual: Type },
    #[error("invalid lookup type: {0}")]
    InvalidLookupType(u8),
}

impl diagnostics::AsDiagnostic for CompileError {
    fn as_diagnostic(&self, source: &str, span: &Span) -> diagnostics::ExprDiagnostic {
        match self {
            CompileError::Undefined(_) => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            CompileError::WrongNumberOfArgs {
                expected: _,
                actual: _,
            } => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            CompileError::NoCallee => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            CompileError::TypeMismatch {
                expected: _,
                actual: _,
            } => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            CompileError::InvalidLookupType(_) => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum RuntimeError {
    #[error("attempting to pop from an empty stack")]
    EmptyStack,
    #[error("expected type {expected} but received {actual}")]
    TypeMismatch { expected: Type, actual: Type },
}

impl diagnostics::AsDiagnostic for RuntimeError {
    fn as_diagnostic(&self, source: &str, span: &Span) -> diagnostics::ExprDiagnostic {
        match self {
            RuntimeError::EmptyStack => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
            RuntimeError::TypeMismatch {
                expected: _,
                actual: _,
            } => diagnostics::ExprDiagnostic {
                code: "".to_string(),
                range: diagnostics::get_range(source, span),
                severity: Some(diagnostics::DiagnosisSeverity::ERROR),
                message: format!("{self}"),
            },
        }
    }
}

pub type ExprErrorS = Spanned<ExprError>;

pub mod diagnostics {
    use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
    use line_col::LineColLookup;

    use crate::{errors::ExprErrorS, span::Span};

    pub fn get_diagnostics(errs: &[ExprErrorS], source: &str) -> Vec<Diagnostic<()>> {
        errs.iter()
            .map(|(err, span)| {
                let a = err.as_diagnostic(source, span);
                let b = a.to_diagnostic(span).with_message(a.message.clone());

                b
            })
            .collect()
    }

    pub trait AsDiagnostic {
        fn as_diagnostic(&self, source: &str, span: &Span) -> ExprDiagnostic;
    }

    #[derive(Debug, Eq, PartialEq, Clone, Default)]
    pub struct ExprDiagnostic {
        pub code: String,

        pub range: ExprDiagnosticRange,

        pub severity: Option<DiagnosisSeverity>,

        pub message: String,
    }

    impl ExprDiagnostic {
        pub fn to_diagnostic(&self, span: &Span) -> codespan_reporting::diagnostic::Diagnostic<()> {
            codespan_reporting::diagnostic::Diagnostic {
                severity: DiagnosisSeverity::ERROR.to_severity(),
                code: Some(self.code.clone()),
                message: self.message.clone(),
                labels: vec![Label::primary((), span.clone())],
                notes: vec![],
            }
        }
    }

    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
    pub struct DiagnosisSeverity(i32);
    #[allow(dead_code)]
    impl DiagnosisSeverity {
        pub const ERROR: DiagnosisSeverity = DiagnosisSeverity(1);
        pub const WARNING: DiagnosisSeverity = DiagnosisSeverity(2);
        pub const INFORMATION: DiagnosisSeverity = DiagnosisSeverity(3);
        pub const HINT: DiagnosisSeverity = DiagnosisSeverity(4);
    }

    impl DiagnosisSeverity {
        fn to_severity(&self) -> Severity {
            match *self {
                DiagnosisSeverity::HINT => Severity::Help,
                DiagnosisSeverity::INFORMATION => Severity::Note,
                DiagnosisSeverity::WARNING => Severity::Warning,
                DiagnosisSeverity::ERROR => Severity::Error,
                _ => panic!("Invalid diagnosis severity: {}", self.0),
            }
        }
    }

    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default)]
    pub struct ExprDiagnosticPosition {
        pub line: u32,
        pub character: u32,
    }

    impl ExprDiagnosticPosition {
        pub fn new(line: u32, character: u32) -> ExprDiagnosticPosition {
            ExprDiagnosticPosition { line, character }
        }
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
    pub struct ExprDiagnosticRange {
        /// The range's start position (inclusive)
        pub start: ExprDiagnosticPosition,
        /// The range's end position (exclusive)
        pub end: ExprDiagnosticPosition,
    }

    impl ExprDiagnosticRange {
        pub fn new(
            start: ExprDiagnosticPosition,
            end: ExprDiagnosticPosition,
        ) -> ExprDiagnosticRange {
            ExprDiagnosticRange { start, end }
        }
    }

    pub fn get_range(source: &str, span: &Span) -> ExprDiagnosticRange {
        ExprDiagnosticRange::new(
            get_position(source, span.start),
            get_position(source, span.end),
        )
    }

    pub fn get_position(source: &str, idx: usize) -> ExprDiagnosticPosition {
        let (line, character) = index_to_position(source, idx);

        ExprDiagnosticPosition::new(line as u32, character as u32)
    }

    /// Map index to position (line, column)
    ///
    /// Line and column are zero based
    pub fn index_to_position(source: &str, index: usize) -> (usize, usize) {
        let lookup = LineColLookup::new(source);

        let (line, char) = lookup.get(index);

        (line - 1, char - 1)
    }

    /// Map position (line, column) to index
    ///
    /// Line and column are zero based
    pub fn position_to_index(source: &str, position: (usize, usize)) -> usize {
        let (line, character) = position;
        let lines = source.split('\n');
        let lines_before = lines.take(line);
        let line_chars_before = lines_before.fold(0usize, |acc, e| acc + e.len() + 1);
        let chars = character;

        line_chars_before + chars
    }

    #[cfg(test)]
    mod index_position_fn_tests {
        use super::*;

        #[test]
        fn it_should_convert_index_to_position() {
            let source = "let a = 123;\nlet b = 456;";

            let index = 17usize;
            let expected_position = (1, 4);

            let index_to_position = index_to_position(source, index);
            let actual_position = index_to_position;

            assert_eq!(expected_position, actual_position);
        }

        #[test]
        fn it_should_convert_position_to_index() {
            let source = "let a = 123;\nlet b = 456;";
            let position = (1, 4);
            let expected_index = 17usize;
            let actual_index = position_to_index(source, position);

            assert_eq!(expected_index, actual_index);
        }

        #[test]
        fn it_should_convert_position_to_index_and_back() {
            let source = "let a = 123;\nlet b = 456;";
            let position = (1, 4);
            let actual_index = position_to_index(source, position);

            assert_eq!(position, index_to_position(source, actual_index));
        }

        #[test]
        fn it_should_convert_position_to_index_and_back_b() {
            let source = "let a = 123;\n{\n    let b = 456;\n}";
            let position = (2, 12);
            let actual_index = position_to_index(source, position);

            assert_eq!(position, index_to_position(source, actual_index));
        }

        #[test]
        fn it_should_convert_position_to_index_b() {
            let source = "let a = 123;\n{\n    let b = 456;\n}";
            let position = (2, 12);
            let actual_index = position_to_index(source, position);

            assert_eq!(27, actual_index);
        }

        #[test]
        fn it_should_convert_position_to_index_c() {
            let source = "let a = 123;\nlet b = 456;\nlet c = 789;";
            let position = (2, 8);
            let actual_index = position_to_index(source, position);

            assert_eq!(34, actual_index);
        }

        #[test]
        fn it_should_convert_position_to_index_d() {
            let source = "let a = 123;\nlet b = 456;\nlet c = 789;\nlet d = 000;";
            let position = (3, 8);
            let actual_index = position_to_index(source, position);

            assert_eq!(47, actual_index);
        }

        #[test]
        fn it_should_convert_position_to_index_e() {
            let source = "let a = 123;\nlet b = 456;\nlet c = 789;\nlet d = 000;\nlet e = 999;";
            let position = (4, 8);
            let actual_index = position_to_index(source, position);

            assert_eq!(60, actual_index);
        }

        #[test]
        fn it_should_convert_position_to_index_f() {
            let source = "let a = 123;\nlet b = 456;\nlet c = 789;\nlet d = 000;\nlet e = 999;\n";
            let position = (4, 8);
            let actual_index = position_to_index(source, position);

            assert_eq!(60, actual_index);
        }
    }

    #[cfg(test)]
    mod error_to_diagnostics_tests {
        use crate::errors::{CompileError, ExprError, LexicalError};

        use super::*;
        use std::ops::Range;

        fn dummy_source() -> &'static str {
            "fn test_function(x: i32) -> i32 { x + 1 }"
        }

        fn dummy_range() -> Span {
            Range { start: 0, end: 5 }
        }

        #[test]
        fn it_converts_lexerror_to_diagnostic() {
            let source = dummy_source();
            let range = dummy_range();
            let error = ExprError::LexError(LexicalError::InvalidToken);
            let diagnostics = get_diagnostics(&[(error, range.clone())], source);

            assert_eq!(diagnostics.len(), 1);
            let diagnostic = &diagnostics[0];
            assert_eq!(diagnostic.code, Some("".to_string()));
            assert_eq!(diagnostic.message, "Invalid token".to_string());
            assert_eq!(diagnostic.severity, Severity::Error);
            assert_eq!(diagnostic.labels.len(), 1);
            assert_eq!(diagnostic.labels[0], Label::primary((), range));
        }

        #[test]
        fn it_converts_compileerror_undefined_to_diagnostic() {
            let source = dummy_source();
            let range = dummy_range();
            let error = ExprError::CompileError(CompileError::Undefined("var".to_string()));
            let diagnostics = get_diagnostics(&[(error, range.clone())], source);

            assert_eq!(diagnostics.len(), 1);
            let diagnostic = &diagnostics[0];
            assert_eq!(diagnostic.code, Some("".to_string()));
            assert_eq!(diagnostic.message, "undefined: var".to_string());
            assert_eq!(diagnostic.severity, Severity::Error);
            assert_eq!(diagnostic.labels.len(), 1);
            assert_eq!(diagnostic.labels[0], Label::primary((), range));
        }

        #[test]
        fn it_converts_compileerror_wrong_number_of_args_to_diagnostic() {
            let source = dummy_source();
            let range = dummy_range();
            let error = ExprError::CompileError(CompileError::WrongNumberOfArgs {
                expected: 2,
                actual: 3,
            });
            let diagnostics = get_diagnostics(&[(error, range.clone())], source);

            assert_eq!(diagnostics.len(), 1);
            let diagnostic = &diagnostics[0];
            assert_eq!(diagnostic.code, Some("".to_string()));
            assert_eq!(
                diagnostic.message,
                "expects 2 arguments but received 3".to_string()
            );
            assert_eq!(diagnostic.severity, Severity::Error);
            assert_eq!(diagnostic.labels.len(), 1);
            assert_eq!(diagnostic.labels[0], Label::primary((), range));
        }
    }
}
