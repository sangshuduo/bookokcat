use std::{env, fs::File, io::stdout};

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::{error, info};
use ratatui::{Terminal, backend::CrosstermBackend};
use simplelog::{LevelFilter, WriteLogger};

// Use modules from the library crate
use bookokcat::event_source::KeyboardEventSource;
use bookokcat::main_app::{App, run_app_with_event_source};
use bookokcat::panic_handler;

fn main() -> Result<()> {
    // Initialize logging with html5ever DEBUG logs filtered out
    WriteLogger::init(
        LevelFilter::Debug,
        simplelog::ConfigBuilder::new()
            .set_max_level(LevelFilter::Debug)
            .add_filter_ignore_str("html5ever")
            .build(),
        File::create("bookokcat.log")?,
    )?;

    let args: Vec<String> = env::args().skip(1).collect();
    if matches!(args.first().map(|s| s.as_str()), Some("--debug-pdf")) {
        let pdf_path = args
            .get(1)
            .context("Usage: bookokcat --debug-pdf <path-to-pdf>")?;
        info!("Running PDF debug mode for {}", pdf_path);
        let result = run_pdf_debug(pdf_path);
        if let Err(err) = &result {
            error!("PDF debug mode failed: {err:?}");
        } else {
            info!("PDF debug mode completed successfully for {}", pdf_path);
        }
        return result;
    }

    // Initialize panic handler only for interactive TUI mode
    panic_handler::initialize_panic_handler();

    info!("Starting Bookokcat EPUB reader");

    // Terminal initialization
    enable_raw_mode().map_err(|e| {
        error!("Failed to enable raw mode: {e}");
        anyhow::anyhow!(
            "Failed to initialize terminal: {e}\n\
             Make sure you are running bookokcat in a terminal, not from a pipe or redirection."
        )
    })?;
    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        error!("Failed to setup terminal: {e}");
        let _ = disable_raw_mode();
        anyhow::anyhow!(
            "Failed to setup terminal: {e}\n\
             Make sure you are running bookokcat in a proper terminal environment."
        )
    })?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let mut event_source = KeyboardEventSource;
    let res = run_app_with_event_source(&mut terminal, &mut app, &mut event_source);

    // Restore terminal state
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = terminal.show_cursor();

    if let Err(err) = res {
        error!("Application error: {err:?}");
        println!("{err:?}");
    }

    info!("Shutting down Bookokcat");
    Ok(())
}

fn run_pdf_debug(pdf_path: &str) -> Result<()> {
    use bookokcat::book_manager::BookManager;
    use bookokcat::pdf_handler::{
        PdfDocument, clear_pdf_progress_callback, set_pdf_progress_callback,
    };
    use std::fs;
    use std::path::Path;

    // Set environment variable to prevent subprocess recursion
    unsafe {
        env::set_var("BOOKOKCAT_DEBUG_PDF_MODE", "1");
    }

    info!("Starting PDF diagnostics for {}", pdf_path);
    println!("PDF debug mode");
    println!("==============");
    println!("Target: {pdf_path}");

    let path = Path::new(pdf_path);
    if !path.exists() {
        anyhow::bail!("PDF not found at path: {pdf_path}");
    }

    let metadata =
        fs::metadata(path).with_context(|| format!("Failed to read metadata for {pdf_path}"))?;
    let size_bytes = metadata.len();
    println!(
        "File size: {} bytes ({:.2} MiB)",
        size_bytes,
        size_bytes as f64 / (1024.0 * 1024.0)
    );

    set_pdf_progress_callback(|message, progress| {
        println!("[progress {:>3}%] {}", progress, message);
    });
    let load_result = PdfDocument::load(pdf_path);
    clear_pdf_progress_callback();

    let pdf_doc = match load_result {
        Ok(doc) => doc,
        Err(err) => {
            println!("PdfDocument::load failed: {err:?}");
            return Err(err);
        }
    };

    println!("PdfDocument::load succeeded.");
    println!("  Reported page count: {}", pdf_doc.page_count());
    println!("  Reported file size: {} bytes", pdf_doc.file_size());

    match pdf_doc.extract_text() {
        Ok(text) => {
            println!(
                "extract_text() succeeded; {} bytes / {} chars",
                text.len(),
                text.chars().count()
            );
            let preview: String = text.chars().take(400).collect();
            if !preview.is_empty() {
                println!("--- Text preview (first 400 chars) ---");
                println!("{preview}");
                println!("--- end preview ---");
            } else {
                println!("extract_text() returned an empty string.");
            }
        }
        Err(err) => {
            println!("extract_text() failed: {err:?}");
        }
    }

    if let Some(parent) = path.parent().and_then(|p| p.to_str()) {
        println!("\nBookManager diagnostics (directory: {parent})");
        let manager = BookManager::new_with_directory(parent);
        println!("  Managed entries discovered: {}", manager.books.len());

        if manager.contains_book(pdf_path) {
            match manager.load_epub(pdf_path) {
                Ok(doc) => {
                    println!(
                        "  BookManager::load_epub succeeded ({} chapters, current index {}).",
                        doc.get_num_chapters(),
                        doc.get_current_chapter()
                    );
                }
                Err(err) => {
                    println!("  BookManager::load_epub error: {err}");
                }
            }
        } else {
            println!("  BookManager did not include this file; check directory scanning rules.");
        }
    } else {
        println!("\nUnable to derive parent directory for BookManager diagnostics.");
    }

    println!("\nDiagnostics complete.");
    info!("Completed PDF diagnostics for {}", pdf_path);
    Ok(())
}
