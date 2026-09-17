use std::collections::HashMap;
use std::path::Path;

use crate::analyzer::{analyze, Diagnostic};
use crate::class_index::ClassIndex;
use crate::symbols::SymbolIndex;

#[derive(Debug)]
pub struct WorkspaceDocument {
    pub text: String,
    pub version: i64,
    pub symbols: SymbolIndex,
    pub classes: ClassIndex,
    pub diagnostics: Vec<Diagnostic>,
}

impl WorkspaceDocument {
    pub fn new(
        version: i64,
        text: String,
    ) -> Self {
        let analysis =
            analyze(&text);

        let mut symbols =
            SymbolIndex::new();

        symbols.rebuild(
            &text,
            &analysis.statements,
        );

        let mut classes = ClassIndex::new();

        classes.rebuild(&analysis.statements);

        Self {
            text,
            version,
            symbols,
            classes,
            diagnostics:
                analysis.diagnostics,
        }
    }

    pub fn update(
        &mut self,
        version: i64,
        text: String,
    ) {
        let analysis =
            analyze(&text);

        self.version = version;
        self.text = text;

        self.symbols.rebuild(
            &self.text,
            &analysis.statements,
        );

        self.classes.rebuild(&analysis.statements);

        self.diagnostics =
            analysis.diagnostics;
    }
}

#[derive(Debug, Default)]
pub struct Workspace {
    documents:
        HashMap<String, WorkspaceDocument>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            documents:
                HashMap::new(),
        }
    }

    pub fn open(
        &mut self,
        uri: String,
        version: i64,
        text: String,
    ) {
        let document =
            WorkspaceDocument::new(
                version,
                text,
            );

        self.documents.insert(
            uri,
            document,
        );
    }

    pub fn open_file(
        &mut self,
        uri: String,
        path: &Path,
    ) -> std::io::Result<()> {
        let text =
            std::fs::read_to_string(path)?;

        let document =
            WorkspaceDocument::new(
                0,
                text,
            );

        self.documents.insert(
            uri,
            document,
        );

        Ok(())
    }

    pub fn update(
        &mut self,
        uri: &str,
        version: i64,
        text: String,
    ) -> bool {
        let Some(document) =
            self.documents.get_mut(uri)
        else {
            return false;
        };

        document.update(
            version,
            text,
        );

        true
    }

    pub fn close(
        &mut self,
        uri: &str,
    ) -> Option<WorkspaceDocument> {
        self.documents.remove(uri)
    }

    pub fn get(
        &self,
        uri: &str,
    ) -> Option<&WorkspaceDocument> {
        self.documents.get(uri)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (&String, &WorkspaceDocument)
    > {
        self.documents.iter()
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }
}