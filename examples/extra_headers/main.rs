use anyhow::Result;
use chromatica::Chromatica;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::Duration;

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

    page.set_extra_headers(HashMap::from([("Test", "Example")]))
        .await?;

    let (mut request_subscriber, mut response_subscriber) = page.subscribe_to_requests().await?;
    let request_task = tokio::spawn(async move {
        while let Some(request) = request_subscriber.next().await {
            let _ = request.continue_request().await;
            println!("request url: {:?}", request.url());
            //Browser won't add extra header for OPTIONS method (preflight request), but you can still intercept it.
            println!("request method: {:?}", request.method());
            let headers = request.headers();
            if headers.contains_key("Test") {
                println!("Test header found: {:?}", headers.get("Test"));
            } else {
                println!("Test header not found");
            }
        }
    });

    let response_task = tokio::spawn(async move {
        while let Some(response) = response_subscriber.next().await {
            let _ = response.continue_response().await;
        }
    });

    let html = to_file_url("extra_headers.html");
    page.navigate(&html, None, None).await?;

    tokio::time::sleep(Duration::from_secs(4)).await;

    page.clear_extra_headers().await?;

    page.reload(None, None).await?;

    tokio::time::sleep(Duration::from_secs(4)).await;

    request_task.abort();
    response_task.abort();

    page.close().await?;

    Ok(())
}
