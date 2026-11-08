//! Tree-sitter incremental parser for extracting syntax nodes and symbols.

use std::collections::HashMap;

/// Supported programming languages for AST parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
    Cpp,
}

impl SupportedLanguage {
    /// Returns the language ID used in DocumentMetadata.
    pub fn language_id(&self) -> u16 {
        match self {
            SupportedLanguage::Rust => 1,
            SupportedLanguage::Python => 2,
            SupportedLanguage::TypeScript => 3,
            SupportedLanguage::JavaScript => 4,
            SupportedLanguage::Go => 5,
            SupportedLanguage::Java => 6,
            SupportedLanguage::Cpp => 7,
        }
    }

    /// Detect language from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(SupportedLanguage::Rust),
            "py" => Some(SupportedLanguage::Python),
            "ts" | "tsx" => Some(SupportedLanguage::TypeScript),
            "js" | "jsx" => Some(SupportedLanguage::JavaScript),
            "go" => Some(SupportedLanguage::Go),
            "java" => Some(SupportedLanguage::Java),
            "cpp" | "cc" | "cxx" | "hpp" | "h" => Some(SupportedLanguage::Cpp),
            _ => None,
        }
    }
}

/// Extracted symbol from AST analysis.
#[derive(Debug, Clone)]
pub struct ExtractedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub byte_range: (usize, usize),
    pub line_range: (usize, usize),
    pub parent_scope: Option<String>,
}

/// Classification of AST symbol types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Enum,
    Variable,
    Constant,
    Module,
    Type,
}

impl SymbolKind {
    /// Returns the AST node type ID used in SearchHit.
    pub fn to_ast_node_type(&self) -> u16 {
        match self {
            SymbolKind::Function | SymbolKind::Method => 1,
            SymbolKind::Class | SymbolKind::Struct => 2,
            SymbolKind::Interface => 3,
            SymbolKind::Variable | SymbolKind::Constant => 4,
            SymbolKind::Enum => 5,
            SymbolKind::Module => 6,
            SymbolKind::Type => 7,
        }
    }
}

/// Manages Tree-sitter parsers for multiple languages.
pub struct ParserPool {
    _parsers: HashMap<SupportedLanguage, ()>,
}

impl ParserPool {
    /// Create a new parser pool. Languages are lazily initialized.
    pub fn new() -> Self {
        Self {
            _parsers: HashMap::new(),
        }
    }

    /// Parse source code and extract symbols.
    ///
    /// NOTE: Tree-sitter language grammars must be loaded from compiled .so/.dll files.
    /// This is a structural placeholder; in production, each grammar is linked via
    /// tree-sitter-<lang> crates.
    pub fn parse_and_extract(
        &mut self,
        _language: SupportedLanguage,
        _source: &[u8],
    ) -> Option<Vec<ExtractedSymbol>> {
        Some(Vec::new())
    }
}

impl Default for ParserPool {
    fn default() -> Self {
        Self::new()
    }
}
