use std::collections::{BTreeMap, BTreeSet};

use crate::AppError;

use super::{AUXILIARY_DOCUMENTS, READER_ROUTES};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ReaderRegistration {
    pub(super) route_id: String,
    pub(super) label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RegisteredDocument {
    pub(super) id: String,
    pub(super) path: String,
    pub(super) reader: Option<ReaderRegistration>,
    auxiliary: bool,
}

impl RegisteredDocument {
    pub(super) fn reader(route_id: &str, label: &str, path: &str) -> Self {
        Self {
            id: route_id.to_owned(),
            path: path.to_owned(),
            reader: Some(ReaderRegistration {
                route_id: route_id.to_owned(),
                label: label.to_owned(),
            }),
            auxiliary: false,
        }
    }

    pub(super) fn auxiliary(id: &str, path: &str) -> Self {
        Self {
            id: id.to_owned(),
            path: path.to_owned(),
            reader: None,
            auxiliary: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct KnowledgeRegistration {
    documents: Vec<RegisteredDocument>,
}

impl KnowledgeRegistration {
    pub(super) fn new(documents: Vec<RegisteredDocument>) -> Result<Self, AppError> {
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        let mut route_ids = BTreeSet::new();
        let mut labels = BTreeSet::new();
        for document in &documents {
            if document.id.is_empty() || !ids.insert(document.id.as_str()) {
                return Err(invalid("duplicate or empty document id"));
            }
            if document.path.is_empty() || !paths.insert(document.path.as_str()) {
                return Err(invalid("duplicate or empty document path"));
            }
            if let Some(reader) = &document.reader {
                if reader.route_id.is_empty() || !route_ids.insert(reader.route_id.as_str()) {
                    return Err(invalid("duplicate or empty reader route id"));
                }
                if reader.label.is_empty() || !labels.insert(reader.label.as_str()) {
                    return Err(invalid("duplicate or empty reader label"));
                }
            }
        }
        Ok(Self { documents })
    }

    pub(super) fn reader_routes(&self) -> impl Iterator<Item = &RegisteredDocument> {
        self.documents
            .iter()
            .filter(|document| document.reader.is_some())
    }

    pub(super) fn auxiliary_documents(&self) -> impl Iterator<Item = &RegisteredDocument> {
        self.documents.iter().filter(|document| document.auxiliary)
    }

    pub(super) fn document_id_by_path(&self, path: &str) -> Option<&str> {
        self.documents
            .iter()
            .find(|document| document.path == path && document.auxiliary)
            .map(|document| document.id.as_str())
    }
}

pub(super) fn static_registration() -> Result<KnowledgeRegistration, AppError> {
    let mut documents = Vec::new();
    let mut index_by_path = BTreeMap::new();
    for (route_id, label, path) in READER_ROUTES {
        index_by_path.insert(path.to_owned(), documents.len());
        documents.push(RegisteredDocument::reader(route_id, label, path));
    }
    for (id, path) in AUXILIARY_DOCUMENTS {
        if let Some(index) = index_by_path.get(path).copied() {
            documents[index].auxiliary = true;
            if documents[index].id != id {
                documents[index].id = id.to_owned();
            }
        } else {
            index_by_path.insert(path.to_owned(), documents.len());
            documents.push(RegisteredDocument::auxiliary(id, path));
        }
    }
    KnowledgeRegistration::new(documents)
}

fn invalid(message: &'static str) -> AppError {
    AppError::invalid_input("knowledge.registration", message)
}
