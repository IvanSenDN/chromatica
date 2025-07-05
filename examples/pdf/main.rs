use anyhow::Result;
use chromatica::{Chromatica, PrintToPDF};
use std::io::{self, Write};
use std::path::PathBuf;

fn asset_path(file: &str) -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let exe_name = exe.file_stem().unwrap().to_str().unwrap();

    let example_dir = exe
        .parent()
        .unwrap() // .../examples/
        .parent()
        .unwrap() // .../debug/
        .parent()
        .unwrap() // .../target/
        .parent()
        .unwrap() // ← project root
        .join("examples")
        .join(exe_name);

    example_dir.join(file)
}

fn to_file_url(file: &str) -> String {
    let path = asset_path(file);
    format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
}

#[tokio::main]
async fn main() -> Result<()> {
    print!("Enter path to save PDF (e.g., output.pdf): ");
    io::stdout().flush()?;

    let mut save_path = String::new();
    io::stdin().read_line(&mut save_path)?;
    let save_path = save_path.trim();

    let mut chromatica = Chromatica::new(None);
    let browser = chromatica.connect(61000, None).await?;
    let page = browser.new_page().await?;

    let html = to_file_url("pdf.html");
    page.navigate(&html, None, None).await?;

    println!("Generating PDF...");
    //Or might be None.
    page.print_to_pdf(Some(save_path), Some(PrintToPDF::default()))
        .await?;
    println!("PDF saved to: {}", save_path);

    Ok(())
}
