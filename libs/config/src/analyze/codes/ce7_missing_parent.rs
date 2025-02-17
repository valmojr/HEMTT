use ariadne::{sources, ColorGenerator, Fmt, Label, Report};
use hemtt_common::reporting::{Annotation, AnnotationLevel, Code, Processed};

use crate::Class;

pub struct MissingParent {
    class: Class,
}

impl MissingParent {
    pub const fn new(class: Class) -> Self {
        Self { class }
    }
}

// TODO: maybe we could have a `did you mean` here without too much trouble?

impl Code for MissingParent {
    fn ident(&self) -> &'static str {
        "CE7"
    }

    fn message(&self) -> String {
        "class's parent is not present".to_string()
    }

    fn label_message(&self) -> String {
        "not present in config".to_string()
    }

    fn help(&self) -> Option<String> {
        self.class.parent().map(|parent| {
            format!(
                "add `class {};` to the config to declare it as external",
                parent.as_str(),
            )
        })
    }

    fn report_generate_processed(&self, processed: &Processed) -> Option<String> {
        let parent = self.class.parent()?;
        let map = processed
            .mapping(
                self.class
                    .name()
                    .expect("parent existed to create error")
                    .span
                    .start,
            )
            .unwrap();
        let token = map.token();
        let parent_map = processed.mapping(parent.span.start).unwrap();
        let parent_token = parent_map.token();
        let mut out = Vec::new();
        let mut colors = ColorGenerator::new();
        let a = colors.next();
        Report::build(
            ariadne::ReportKind::Error,
            token.position().path().as_str(),
            map.original_column(),
        )
        .with_code(self.ident())
        .with_message(self.message())
        .with_label(
            Label::new((
                parent_token.position().path().to_string(),
                parent_token.position().start().0..parent_token.position().end().0,
            ))
            .with_message(self.label_message())
            .with_color(a),
        )
        .with_help(format!(
            "add `class {};` to the config to declare it as external",
            parent.as_str().fg(a),
        ))
        .finish()
        .write_for_stdout(sources(processed.sources_adrianne()), &mut out)
        .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    fn ci_generate_processed(&self, processed: &Processed) -> Vec<Annotation> {
        let map = processed
            .mapping(self.class.parent().unwrap().span.start)
            .unwrap();
        let map_file = processed.source(map.source()).unwrap();
        vec![self.annotation(
            AnnotationLevel::Error,
            map_file.0.as_str().to_string(),
            map.original(),
        )]
    }

    #[cfg(feature = "lsp")]
    fn generate_processed_lsp(&self, processed: &Processed) -> Vec<(vfs::VfsPath, Diagnostic)> {}
}
