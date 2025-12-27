use anyhow::{Context, Result};
use oxidize_pdf::parser::{PdfDocument as OxidizePdfDocument, PdfReader};
use std::fs;
use std::path::Path;

fn suppress_stderr<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    #[cfg(unix)]
    unsafe {
        let null_fd = libc::open(b"/dev/null\0".as_ptr() as *const i8, libc::O_WRONLY);
        if null_fd < 0 {
            return f();
        }
        let old_stderr = libc::dup(libc::STDERR_FILENO);
        libc::dup2(null_fd, libc::STDERR_FILENO);
        libc::close(null_fd);

        let result = f();

        libc::dup2(old_stderr, libc::STDERR_FILENO);
        libc::close(old_stderr);

        result
    }

    #[cfg(not(unix))]
    f()
}

fn can_extract_text(pdf_path: &str) -> Result<bool> {
    let path_owned = pdf_path.to_string();

    let handle = std::thread::spawn(move || {
        suppress_stderr(|| match PdfReader::open(&path_owned) {
            Ok(reader) => {
                let pdf_doc = OxidizePdfDocument::new(reader);

                match pdf_doc.extract_text() {
                    Ok(text_pages) => {
                        for page in text_pages.iter() {
                            if !page.text.is_empty() {
                                return Some(true);
                            }
                        }
                        Some(false)
                    }
                    Err(_) => Some(false),
                }
            }
            Err(_) => Some(false),
        })
    });

    match handle.join() {
        Ok(Some(result)) => Ok(result),
        Ok(None) => Ok(false),
        Err(_) => {
            eprintln!("Thread panicked while processing: {}", pdf_path);
            Ok(false)
        }
    }
}

fn main() -> Result<()> {
    let ebook_dir = "/Users/sangshuduo/OneDrive - huski.ai/ebook";

    if !Path::new(ebook_dir).exists() {
        eprintln!("Directory not found: {}", ebook_dir);
        return Ok(());
    }

    let mut pdf_files: Vec<_> = fs::read_dir(ebook_dir)
        .context("Failed to read ebook directory")?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().map_or(false, |ext| ext == "pdf") {
                    path.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .collect();

    pdf_files.sort();

    println!(
        "Found {} PDF files. Testing extraction...\n",
        pdf_files.len()
    );

    for (index, pdf_path) in pdf_files.iter().enumerate() {
        let filename = Path::new(pdf_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        print!(
            "[{}/{}] Testing: {} ... ",
            index + 1,
            pdf_files.len(),
            filename
        );
        std::io::Write::flush(&mut std::io::stdout()).ok();

        match can_extract_text(pdf_path) {
            Ok(true) => {
                println!("✓ SUCCESS - Can extract readable text!");
                println!("\nPath: {}\n", pdf_path);
                return Ok(());
            }
            Ok(false) => {
                println!("✗ No extractable text");
            }
            Err(e) => {
                println!("✗ Error: {}", e);
            }
        }
    }

    println!("\nNo PDF files with extractable text found.");
    Ok(())
}
