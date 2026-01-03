use local_search::DocumentRequest;
#[cfg(feature = "cli")]
use serde_json;
use std::path::Path;
use log::{debug, info};

#[cfg(feature = "cli")]
/// Ingestor that processes JSON files containing document arrays.
/// Each JSON file should contain an array of [`DocumentRequest`] structs.
pub struct JsonFileIngestor {
    pub indexer: Box<dyn local_search::DocumentIndexer>,
}

#[cfg(feature = "cli")]
impl JsonFileIngestor {

    /// Creates a new JSON file ingestor with the specified document indexer.
    pub fn new(indexer: Box<dyn local_search::DocumentIndexer>) -> Self {
        JsonFileIngestor { indexer }
    }

    /// Ingests JSON files from a file or directory path.
    pub fn ingest(&self, path_str: &str) -> anyhow::Result<()> {
        let path = Path::new(path_str);

        std::fs::metadata(&path).expect("Path does not exist");
        info!("Starting ingestion with path: {}", path_str);
        
        if path.is_dir() {
            let mut processed_files = 0;
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let file_path = entry.path();
                debug!("Processing file: {:?}", file_path);
                if file_path.is_file() && 
                   file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                    self.process_json_file(&file_path)?;
                    processed_files += 1;
                }
                else{
                    debug!("Skipping non-JSON file: {:?}", file_path);
                }
            }
            info!("Processed {} JSON files.", processed_files);
        } else {
            self.process_json_file(path)?;
            info!("Processed single JSON file: {:?}", path);
        }
        
        Ok(())
    }
    
    fn process_json_file(&self, file_path: &Path) -> anyhow::Result<()> {
        let data = std::fs::read_to_string(file_path)?;
        let doc_requests: Vec<DocumentRequest> = serde_json::from_str(&data)?;
        for doc_request in doc_requests {
            self.indexer.upsert_document(doc_request)?;
            info!("Indexed document from file: {:?}", file_path);
        }
        Ok(())
    }
}

/// Ingestor that processes raw text files with custom filtering.
pub struct RawFileIngestor {
    pub indexer: Box<dyn local_search::DocumentIndexer>,
}

impl RawFileIngestor {

    /// Creates a new raw file ingestor with the specified document indexer.
    pub fn new(indexer: Box<dyn local_search::DocumentIndexer>) -> Self {
        RawFileIngestor { indexer }
    }

    /// Ingests raw files from a path using a custom file validation function.
    pub fn ingest<F>(&self, path_str: &str, valid_file_fn: F) -> anyhow::Result<()>
    where
        F: Fn(&Path) -> bool,
    {
        let path = Path::new(path_str);

        std::fs::metadata(&path).expect("Path does not exist");
        info!("Starting ingestion with path: {}", path_str);
        
        if path.is_dir() {
            let mut processed_files = 0;
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let file_path = entry.path();
                debug!("Processing file: {:?}", file_path);
                if file_path.is_file() && valid_file_fn(&file_path) {
                    self.process_file(&file_path)?;
                    processed_files += 1;
                }
                else{
                    debug!("Skipping non-file entry: {:?}", file_path);
                }
            }
            info!("Processed {} files.", processed_files);
        } else {
            self.process_file(path)?;
            info!("Processed single file: {:?}", path);
        }
        
        Ok(())
    }
    
    fn process_file(&self, file_path: &Path) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let doc_request = DocumentRequest {
            path: file_path.to_string_lossy().to_string(),
            content: content,
            metadata: None,
        };
        self.indexer.upsert_document(doc_request)?;
        info!("Indexed document from file: {:?}", file_path);
        Ok(())
    }
}