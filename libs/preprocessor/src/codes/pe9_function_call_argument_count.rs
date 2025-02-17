use ariadne::{sources, ColorGenerator, Fmt, Label, Report, ReportKind};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Token};
use tracing::error;

use crate::defines::Defines;

#[allow(unused)]
/// Tried to call a [`FunctionDefinition`](crate::context::FunctionDefinition) with the wrong number of arguments
pub struct FunctionCallArgumentCount {
    /// The [`Token`] that was found
    pub(crate) token: Box<Token>,
    /// The number of arguments that were expected
    pub(crate) expected: usize,
    /// The number of arguments that were found
    pub(crate) got: usize,
    /// Similar defines
    pub(crate) similar: Vec<String>,
    /// defined
    pub(crate) defined: (Token, Vec<Token>),
}

impl FunctionCallArgumentCount {
    pub fn new(token: Box<Token>, expected: usize, got: usize, defines: &Defines) -> Self {
        Self {
            expected,
            got,
            similar: defines
                .similar_values(token.symbol().to_string().trim())
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            defined: {
                let (t, d) = defines
                    .get_readonly(token.symbol().to_string().trim())
                    .unwrap();
                (
                    t.as_ref().clone(),
                    d.as_function()
                        .unwrap()
                        .clone()
                        .args()
                        .iter()
                        .map(|a| a.as_ref().clone())
                        .collect(),
                )
            },
            token,
        }
    }
}

impl Code for FunctionCallArgumentCount {
    fn ident(&self) -> &'static str {
        "PE9"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!(
            "function call with incorrect number of arguments, expected `{}` got `{}`",
            self.expected, self.got
        )
    }

    fn label_message(&self) -> String {
        format!(
            "incorrect argument count, expected `{}` got `{}`",
            self.expected, self.got,
        )
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn report_generate(&self) -> Option<String> {
        let mut colors = ColorGenerator::default();
        let mut out = Vec::new();
        let span = self.token.position().span();
        let a = colors.next();
        let mut report = Report::build(
            ReportKind::Error,
            self.token.position().path().as_str(),
            span.start,
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                self.token.position().path().to_string(),
                span.start..span.end,
            ))
            .with_color(a)
            .with_message(format!(
                "called with {} argument{} here",
                self.got,
                if self.got == 1 { "" } else { "s" }
            )),
        )
        .with_label(
            Label::new((
                self.defined.0.position().path().to_string(),
                self.defined.0.position().start().0..self.defined.0.position().end().0,
            ))
            .with_color(a)
            .with_message(format!(
                "defined here with {} argument{}",
                self.defined.1.len(),
                if self.defined.1.len() == 1 { "" } else { "s" }
            )),
        );
        if !self.similar.is_empty() {
            report = report.with_help(format!(
                "did you mean `{}`",
                self.similar
                    .iter()
                    .map(|dym| format!("{}", dym.fg(a)))
                    .collect::<Vec<_>>()
                    .join("`, `")
            ));
        }
        if let Err(e) = report.finish().write_for_stdout(
            sources(vec![
                (
                    self.token.position().path().to_string(),
                    self.token
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
                (
                    self.defined.0.position().path().to_string(),
                    self.defined
                        .0
                        .position()
                        .path()
                        .read_to_string()
                        .unwrap_or_default(),
                ),
            ]),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return None;
        }
        Some(String::from_utf8(out).unwrap_or_default())
    }

    fn ci_generate(&self) -> Vec<Annotation> {
        vec![self.annotation(
            AnnotationLevel::Error,
            self.token.position().path().as_str().to_string(),
            self.token.position(),
        )]
    }

    #[cfg(feature = "lsp")]
    fn generate_lsp(&self) -> Option<(VfsPath, Diagnostic)> {
        let Some(path) = self.token.position().path() else {
            return None;
        };
        Some((
            path.clone(),
            self.diagnostic(Range {
                start: self.token.position().start().to_lsp(),
                end: self.token.position().end().to_lsp(),
            }),
        ))
    }
}
