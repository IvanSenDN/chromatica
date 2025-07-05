use anyhow::Result;
use chromatica::Chromatica;
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

    let (mut request_subscriber, mut response_subscriber) = page.subscribe_to_requests().await?;
    let request_task = tokio::spawn(async move {
        while let Some(request) = request_subscriber.next().await {
            //https://chromedevtools.github.io/devtools-protocol/tot/Network/#type-ResourceType
            if request.resource_type() == "Image" {
                let _ = request.abort().await;
            } else {
                let _ = request.continue_request().await;
            }
        }
    });

    let response_task = tokio::spawn(async move {
        while let Some(response) = response_subscriber.next().await {
            //https://chromedevtools.github.io/devtools-protocol/tot/Network/#type-ResourceType
            if response.resource_type() == "Image" {
                let _ = response.abort().await;
            } else {
                let _ = response.continue_response().await;
            }
        }
    });

    let html = to_file_url("request_interception.html");
    let (_, response) = tokio::join!(
        page.navigate(&html, None, None),
        page.wait_for_response(
            |response| response.url() == "https://jsonplaceholder.typicode.com/todos/1",
            None
        )
    );
    let response = response.unwrap();
    println!("url: {:?}", response.url());
    println!("text: {:?}", response.text().unwrap());
    println!("json: {:?}", response.json().unwrap());
    println!("status code: {:?}", response.response_status_code());
    println!("status text: {:?}", response.response_status_text());
    println!("response headers: {:?}", response.response_headers());

    request_task.abort();
    response_task.abort();

    tokio::time::sleep(Duration::from_secs(5)).await;

    page.reload(None, None).await?;

    Ok(())
}
