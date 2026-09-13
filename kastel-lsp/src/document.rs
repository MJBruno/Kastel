use crate::symbols::SymbolIndex;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: String,
    pub language_id: String,
    pub version: i64,
    pub text: String,
    pub symbols: SymbolIndex,
}

#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<String, Document>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn open(&mut self, uri: String, language_id: String, version: i64, text: String) {
        let document = Document {
            uri: uri.clone(),
            language_id,
            version,
            text,
            symbols: SymbolIndex::new(),
        };

        self.documents.insert(uri, document);
    }

    pub fn get(&self, uri: &str) -> Option<&Document> {
        self.documents.get(uri)
    }

    pub fn get_mut(&mut self, uri: &str) -> Option<&mut Document> {
        self.documents.get_mut(uri)
    }

    pub fn close(&mut self, uri: &str) -> Option<Document> {
        self.documents.remove(uri)
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    pub fn update(&mut self, uri: &str, version: i64, text: String) -> bool {
        let Some(document) = self.documents.get_mut(uri) else {
            return false;
        };

        document.version = version;
        document.text = text;

        true
    }
}

pub fn parse_did_open(params: &Value) -> Option<(String, String, i64, String)> {
    let document = params.get("textDocument")?;

    let uri = document.get("uri")?.as_str()?.to_owned();

    let language_id = document.get("languageId")?.as_str()?.to_owned();

    let version = document.get("version")?.as_i64()?;

    let text = document.get("text")?.as_str()?.to_owned();

    Some((uri, language_id, version, text))
}

pub fn parse_did_change(params: &Value) -> Option<(String, i64, String)> {
    let document = params.get("textDocument")?;

    let uri = document.get("uri")?.as_str()?.to_owned();

    let version = document.get("version")?.as_i64()?;

    let changes = params.get("contentChanges")?.as_array()?;

    let change = changes.first()?;

    let text = change.get("text")?.as_str()?.to_owned();

    Some((uri, version, text))
}
