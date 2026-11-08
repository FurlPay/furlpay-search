//! Full-text BM25 index layer backed by Tantivy.

use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, IndexRecordOption, NumericOptions, Schema, TextFieldIndexing, TextOptions, Value},
    Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument,
    doc,
};
use anyhow::Result;
use std::path::Path;

/// Field handles for the Tantivy schema.
pub struct FieldHandles {
    pub path: Field,
    pub content: Field,
    pub symbols: Field,
    pub language: Field,
    pub doc_id: Field,
}

/// Wrapper around a Tantivy full-text index for BM25 scoring.
pub struct BM25Index {
    index: Index,
    reader: IndexReader,
    fields: FieldHandles,
    schema: Schema,
}

impl BM25Index {
    /// Create a new in-memory BM25 index.
    pub fn new_in_memory() -> Result<Self> {
        let (schema, fields) = Self::build_schema();

        let index = Index::create_in_ram(schema.clone());
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            fields,
            schema,
        })
    }

    /// Create a persistent BM25 index at the given directory.
    pub fn new_persistent(dir: &Path) -> Result<Self> {
        let (schema, fields) = Self::build_schema();

        std::fs::create_dir_all(dir)?;
        let index = Index::create_in_dir(dir, schema.clone())?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            fields,
            schema,
        })
    }

    fn build_schema() -> (Schema, FieldHandles) {
        let mut schema_builder = Schema::builder();

        let text_field_indexing = TextFieldIndexing::default()
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_field_indexing);
        let text_stored = text_options.clone().set_stored();

        let path = schema_builder.add_text_field("path", text_stored.clone());
        let content = schema_builder.add_text_field("content", text_options.clone());
        let symbols = schema_builder.add_text_field("symbols", text_options);
        let language = schema_builder.add_text_field("language", text_stored);
        let doc_id = schema_builder.add_u64_field(
            "doc_id",
            NumericOptions::default().set_indexed().set_stored(),
        );
        let schema = schema_builder.build();

        (
            schema,
            FieldHandles {
                path,
                content,
                symbols,
                language,
                doc_id,
            },
        )
    }

    /// Get a writer for batch indexing.
    pub fn writer(&self, heap_size: usize) -> Result<IndexWriter> {
        Ok(self.index.writer(heap_size)?)
    }

    /// Index a document.
    pub fn add_document(
        &self,
        writer: &IndexWriter,
        id: u32,
        file_path: &str,
        content: &str,
        symbols_text: &str,
        language: &str,
    ) -> Result<()> {
        writer.add_document(doc!(
            self.fields.doc_id => id as u64,
            self.fields.path => file_path,
            self.fields.content => content,
            self.fields.symbols => symbols_text,
            self.fields.language => language,
        ))?;
        Ok(())
    }

    /// Search for documents matching a query string.
    pub fn search(&self, query_str: &str, max_results: usize) -> Result<Vec<(f32, u64)>> {
        let searcher = self.reader.searcher();
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.fields.content, self.fields.symbols]);
        let query = query_parser.parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(max_results))?;

        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_addr) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_addr)?;
            if let Some(id_val) = doc.get_first(self.fields.doc_id) {
                if let Some(id) = id_val.as_u64() {
                    results.push((score, id));
                }
            }
        }
        Ok(results)
    }

    /// Reload the reader to pick up committed changes.
    pub fn reload(&self) -> Result<()> {
        self.reader.reload()?;
        Ok(())
    }

    /// Access the underlying schema.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Access the field handles.
    pub fn fields(&self) -> &FieldHandles {
        &self.fields
    }
}
