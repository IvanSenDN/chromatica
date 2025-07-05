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

    let html = to_file_url("alert.html");
    page.navigate(&html, None, None).await?;

    let element = page.wait_for_selector("button", None).await?;
    let (_, dialog) = tokio::join!(
        element.click(),
        page.wait_for_js_dialog(|dialog| dialog.message() == "This is an alert dialog", None,)
    );
    dialog.unwrap().accept(None).await?;
    println!("Dialog accepted");

    Ok(())
}
