use anyhow::Result;
use chromatica::Chromatica;
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
    print!("Enter path to save screenshot (e.g., output.png): ");
    io::stdout().flush()?;

    let mut save_path = String::new();
    io::stdin().read_line(&mut save_path)?;
    let save_path = save_path.trim();

    let mut chromatica = Chromatica::new(None);
    let browser = chromatica.connect(61000, None).await?;
    let page = browser.new_page().await?;

    let html = to_file_url("screenshot.html");
    page.navigate(&html, None, None).await?;

    println!("Generating screenshot...");
    //Or might be None.
    page.screenshot(Some(save_path), None, None, None).await?;
    println!("Screenshot saved to: {}", save_path);

    print!("Enter path to save element screenshot (e.g., output.png): ");
    io::stdout().flush()?;
    let mut save_path = String::new();
    io::stdin().read_line(&mut save_path)?;
    let save_path = save_path.trim();

    let element = page.wait_for_selector(".card", None).await?;
    element
        .screenshot(Some(save_path), None, None, None)
        .await?;
    println!("Element screenshot saved to: {}", save_path);

    Ok(())
}
