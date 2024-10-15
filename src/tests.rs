use crate::website_files::*;

#[test]
fn t_builds_raw_url() {
    let mut instance = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/js",
        "animations.js",
    );

    instance.build_url();

    assert_eq!(instance.url, String::from("https://raw.githubusercontent.com/OneilNvM/rl-hours-tracker/refs/heads/master/website/js/animations.js"));
}

#[test]
fn t_builds_image_url() {
    let mut instance = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/images",
        "rl-icon-black.png",
    );

    instance.build_image_url();

    assert_eq!(instance.url, String::from("https://github.com/OneilNvM/rl-hours-tracker/blob/master/website/images/rl-icon-black.png"));
}

#[tokio::test]
async fn t_sends_request() -> Result<(), reqwest::Error> {
    let mut instance = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/js",
        "animations.js",
    );
    instance.build_url();

    let response = send_request(&instance.url).await;

    let text = response.text().await?;

    println!("Output:\n{}", text);

    Ok(())
}

#[tokio::test]
async fn t_handle_raw_response() {
    let mut instance = Github::new("OneilNvM", "rl-hours-tracker", "master", "src", "main.rs");

    instance.build_url();

    let response = send_request(&instance.url).await;

    let text = response.text().await;

    assert!(text.is_ok())
}

#[tokio::test]
async fn t_handle_image_response() {
    let mut instance = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/images",
        "rl-icon-black.png",
    );

    instance.build_image_url();

    let response = send_request(&instance.url).await;

    let text = response.bytes().await;

    assert!(text.is_ok())
}
