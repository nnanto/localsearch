use localsearch::DocumentRequest;
use log::{debug, info};

use serde_json;
use std::path::Path;

pub struct IngestionResult {
    pub indexed_count: usize,
    pub failed_count: usize,
    pub failed_files: Vec<String>,
    pub total_document_count: i64,
}

impl IngestionResult {
    pub fn new() -> Self {
        IngestionResult {
            indexed_count: 0,
            failed_count: 0,
            failed_files: Vec::new(),
            total_document_count: 0,
        }
    }

    fn add_success(&mut self) {
        self.indexed_count += 1;
    }

    fn add_failure(&mut self, file_path: &Path, error: &anyhow::Error) {
        self.failed_count += 1;
        self.failed_files
            .push(file_path.to_string_lossy().to_string() + ": " + &error.to_string());
        debug!("Failed to process file {:?}: {}", file_path, error);
    }
}

/// Common file processing logic shared by both ingestors
fn process_files<F>(
    path_str: &str,
    should_process_file: F,
    process_single_file: impl Fn(&Path) -> anyhow::Result<()>,
) -> anyhow::Result<IngestionResult>
where
    F: Fn(&Path) -> bool,
{
    let path = Path::new(path_str);
    let mut result = IngestionResult::new();

    std::fs::metadata(path).expect("Path does not exist");
    info!("Starting ingestion with path: {}", path_str);

    if path.is_dir() {
        // First pass: count eligible files for progress reporting
        let eligible_files: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|file_path| file_path.is_file() && should_process_file(file_path))
            .collect();

        let total_files = eligible_files.len();
        println!("Found {} files to process", total_files);

        // Second pass: process files with progress
        for (index, file_path) in eligible_files.iter().enumerate() {
            let progress_percent = ((index + 1) as f32 / total_files as f32 * 100.0) as u32;
            println!(
                "Processing file {} of {} ({}%): {}",
                index + 1,
                total_files,
                progress_percent,
                file_path.file_name().unwrap_or_default().to_string_lossy()
            );

            match process_single_file(file_path) {
                Ok(_) => {
                    result.add_success();
                    debug!("✓ Successfully indexed: {:?}", file_path);
                }
                Err(e) => {
                    result.add_failure(file_path, &e);
                    println!("✗ Failed to process: {:?} - {}", file_path, e);
                }
            }
        }

        println!(
            "Completed processing {} files ({} succeeded, {} failed)",
            result.indexed_count + result.failed_count,
            result.indexed_count,
            result.failed_count
        );
    } else if should_process_file(path) {
        println!("Processing single file: {}", path.display());
        match process_single_file(path) {
            Ok(_) => {
                result.add_success();
                println!("✓ Successfully processed: {:?}", path);
            }
            Err(e) => {
                result.add_failure(path, &e);
                println!("✗ Failed to process: {:?} - {}", path, e);
            }
        }
    }

    Ok(result)
}

fn update_total_document_count(
    indexer: &dyn localsearch::DocumentIndexer,
    ingestion_result: &mut IngestionResult,
) {
    match indexer.stats() {
        Ok(count) => ingestion_result.total_document_count = count,
        Err(e) => {
            debug!("Failed to retrieve document count: {}", e);
            ingestion_result.total_document_count = -1;
        }
    }
}

/// Ingestor that processes JSON files containing document arrays.
/// Each JSON file should contain an array of [`DocumentRequest`] structs.
pub struct JsonFileIngestor {
    pub indexer: Box<dyn localsearch::DocumentIndexer>,
}

impl JsonFileIngestor {
    /// Creates a new JSON file ingestor with the specified document indexer.
    pub fn new(indexer: Box<dyn localsearch::DocumentIndexer>) -> Self {
        JsonFileIngestor { indexer }
    }

    /// Ingests JSON files from a file or directory path.
    pub fn ingest(&self, path_str: &str) -> anyhow::Result<IngestionResult> {
        let should_process_file =
            |file_path: &Path| file_path.extension().and_then(|s| s.to_str()) == Some("json");

        let process_single_file =
            |file_path: &Path| -> anyhow::Result<()> { self.process_json_file(file_path) };

        let mut r = process_files(path_str, should_process_file, process_single_file)?;
        update_total_document_count(self.indexer.as_ref(), &mut r);
        Ok(r)
    }

    fn process_json_file(&self, file_path: &Path) -> anyhow::Result<()> {
        let data = std::fs::read_to_string(file_path)?;
        let doc_requests: Vec<DocumentRequest> = serde_json::from_str(&data)?;
        for doc_request in doc_requests {
            self.indexer.upsert_document(doc_request)?;
        }
        Ok(())
    }
}

/// Ingestor that processes raw text files with custom filtering.
pub struct RawFileIngestor {
    pub indexer: Box<dyn localsearch::DocumentIndexer>,
}

impl RawFileIngestor {
    /// Creates a new raw file ingestor with the specified document indexer.
    pub fn new(indexer: Box<dyn localsearch::DocumentIndexer>) -> Self {
        RawFileIngestor { indexer }
    }

    /// Ingests raw files from a path using a custom file validation function.
    pub fn ingest<F>(&self, path_str: &str, valid_file_fn: F) -> anyhow::Result<IngestionResult>
    where
        F: Fn(&Path) -> bool,
    {
        let process_single_file =
            |file_path: &Path| -> anyhow::Result<()> { self.process_file(file_path) };

        let mut r = process_files(path_str, valid_file_fn, process_single_file)?;
        update_total_document_count(self.indexer.as_ref(), &mut r);
        Ok(r)
    }

    fn process_file(&self, file_path: &Path) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let doc_request = DocumentRequest {
            path: file_path.to_string_lossy().to_string(),
            content,
            metadata: None,
        };
        self.indexer.upsert_document(doc_request)?;
        Ok(())
    }
}
