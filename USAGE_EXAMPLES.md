# Usage Examples

This file contains clean, practical examples of how to use the PDF extraction and bookmark generation functionality.

## Single PDF Processing

### Extract and Generate Bookmark
```rust
use auto_bookmark::generation::pdf_workflow::pdf_to_bookmark_simple;

fn process_single_pdf() -> Result<(), Box<dyn std::error::Error>> {
    let result = pdf_to_bookmark_simple("input/document.pdf", "output/bookmark.pdf")?;
    println!("Success: {}", result.summary());
    Ok(())
}
```

### Extract Images Only
```rust
use auto_bookmark::generation::extract_pdf_images::extract_recto_verso_pairs_simple;

fn extract_only() -> Result<(), Box<dyn std::error::Error>> {
    let pairs = extract_recto_verso_pairs_simple("input/document.pdf")?;
    
    for (i, pair) in pairs.iter().enumerate() {
        println!("Pair {}: {} -> {}", i + 1, pair.recto_path, pair.verso_path);
    }
    Ok(())
}
```

## Multiple PDFs Processing

### Batch Extract Images
```rust
use auto_bookmark::generation::pdf_workflow::batch_extract_images_simple;

fn batch_extract() -> Result<(), Box<dyn std::error::Error>> {
    let pdfs = vec!["input/doc1.pdf", "input/doc2.pdf", "input/doc3.pdf"];
    let results = batch_extract_images_simple(&pdfs)?;
    
    for (pdf_name, pairs) in results {
        println!("{}: {} pairs extracted", pdf_name, pairs.len());
    }
    Ok(())
}
```

### Batch Process to Bookmarks
```rust
use auto_bookmark::generation::pdf_workflow::batch_process_pdfs_simple;

fn batch_process() -> Result<(), Box<dyn std::error::Error>> {
    let pdfs = vec!["input/doc1.pdf", "input/doc2.pdf"];
    let results = batch_process_pdfs_simple(&pdfs, "output/bookmarks")?;
    
    for result in results {
        println!("Processed: {}", result.summary());
    }
    Ok(())
}
```

## Manual PDF Generation

### From Existing Images
```rust
use auto_bookmark::generation::generate_pdf::{generate_pdf_simple, ImageInfo};

fn manual_generation() -> Result<(), Box<dyn std::error::Error>> {
    let images = vec![
        ImageInfo {
            recto_path: "images/page1_front.png".to_string(),
            verso_path: "images/page1_back.png".to_string(),
        },
        ImageInfo {
            recto_path: "images/page2_front.png".to_string(),
            verso_path: "images/page2_back.png".to_string(),
        },
    ];
    
    let pdf_data = generate_pdf_simple(&images)?;
    std::fs::write("output/manual.pdf", pdf_data)?;
    println!("Manual PDF generated");
    Ok(())
}
```

## Advanced Usage

### Check PDF Before Processing
```rust
use auto_bookmark::generation::extract_pdf_images::PdfImageExtractor;

fn check_pdf_first() -> Result<(), Box<dyn std::error::Error>> {
    let extractor = PdfImageExtractor::new();
    let page_count = extractor.get_page_count("input/document.pdf")?;
    
    if page_count % 2 != 0 {
        return Err("PDF has odd number of pages - cannot create recto-verso pairs".into());
    }
    
    println!("PDF is valid: {} pages, {} pairs possible", page_count, page_count / 2);
    
    // Proceed with extraction...
    let pairs = extractor.extract_recto_verso_pairs("input/document.pdf")?;
    println!("Extracted {} pairs", pairs.len());
    Ok(())
}
```

### Convert Extracted Pairs to ImageInfo
```rust
use auto_bookmark::generation::{
    extract_pdf_images::extract_recto_verso_pairs_simple,
    generate_pdf::{generate_pdf_simple, ImageInfo},
};

fn extract_then_generate() -> Result<(), Box<dyn std::error::Error>> {
    // Extract from PDF
    let pairs = extract_recto_verso_pairs_simple("input/source.pdf")?;
    
    // Convert to ImageInfo
    let images: Vec<ImageInfo> = pairs.into_iter().map(|pair| pair.into()).collect();
    
    // Generate new PDF
    let pdf_data = generate_pdf_simple(&images)?;
    std::fs::write("output/converted.pdf", pdf_data)?;
    
    Ok(())
}
```

## Error Handling

### Robust Processing
```rust
use auto_bookmark::generation::pdf_workflow::pdf_to_bookmark_simple;

fn robust_processing(pdf_files: &[&str]) {
    for pdf_file in pdf_files {
        match pdf_to_bookmark_simple(pdf_file, &format!("output/{}_bookmark.pdf", pdf_file)) {
            Ok(result) => println!("✓ {}: {}", pdf_file, result.summary()),
            Err(e) => println!("✗ {}: {}", pdf_file, e),
        }
    }
}
```

## Directory Processing

### Process All PDFs in Directory
```rust
use std::fs;
use auto_bookmark::generation::pdf_workflow::pdf_to_bookmark_simple;

fn process_directory(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir_path)?;
    let mut processed = 0;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "pdf") {
            let output_path = format!("output/{}_bookmark.pdf", 
                path.file_stem().unwrap().to_string_lossy());
            
            match pdf_to_bookmark_simple(&path, output_path) {
                Ok(_) => {
                    processed += 1;
                    println!("✓ Processed: {:?}", path.file_name());
                }
                Err(e) => println!("✗ Failed {:?}: {}", path.file_name(), e),
            }
        }
    }
    
    println!("Successfully processed {} PDFs", processed);
    Ok(())
}
```

## Constants Configuration

The system uses these constants (defined in `extract_pdf_images.rs`):

```rust
const OUTPUT_DIR: &str = "extracted_images";  // Where images are saved
const DPI: u32 = 300;                        // Image quality
const IMAGE_EXTENSION: &str = "png";          // Always PNG format
```

Images are saved with names like:
- `document_page001_recto.png`
- `document_page002_verso.png`
- `document_page003_recto.png`
- `document_page004_verso.png`

## Running Examples

Make sure to compile with the `ssr` feature for full functionality:

```bash
# Build with PDF processing support
cargo build --features ssr

# Run with PDF processing support
cargo run --features ssr

# Without ssr feature, uses fallback implementations
cargo run
```
