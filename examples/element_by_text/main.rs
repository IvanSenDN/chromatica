use anyhow::Result;
use chromatica::Chromatica;
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
    let mut chromatica = Chromatica::new(None);
    let browser = chromatica.connect(61000, None).await?;
    let page = browser.new_page().await?;

    let html = to_file_url("element_by_text.html");
    page.navigate(&html, None, None).await?;

    let element = page.wait_for_selector("text(Spawn iframe)", None).await?;
    element.click().await?;
    println!("Spawn iframe button clicked");

    let element = page.wait_for_selector("text(Hello, World)", None).await?;
    let attribute = element.attributes().await?;
    println!("id of text element: {:?}", attribute.get("id"));

    Ok(())
}
