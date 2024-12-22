use crate::{date_binary_search, website_files::*};

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

    assert_eq!(instance.get_url(), String::from("https://raw.githubusercontent.com/OneilNvM/rl-hours-tracker/refs/heads/master/website/js/animations.js"));
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

    assert_eq!(instance.get_url(), String::from("https://github.com/OneilNvM/rl-hours-tracker/blob/master/website/images/rl-icon-black.png"));
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

    let response = send_request(&instance.get_url()).await;

    let text = response.text().await?;

    println!("Output:\n{}", text);

    Ok(())
}

#[tokio::test]
async fn t_handle_raw_response() {
    let mut instance = Github::new("OneilNvM", "rl-hours-tracker", "master", "src", "main.rs");

    instance.build_url();

    let response = send_request(&instance.get_url()).await;

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

    let response = send_request(&instance.get_url()).await;

    let text = response.bytes().await;

    assert!(text.is_ok())
}

#[test]
fn t_date_binary_search() {
    let split_newline: Vec<&str> = vec![
        "2024-09-15 58s",
        "2024-09-15 890s",
        "2024-09-16 2890s",
        "2024-09-16 1589s",
        "2024-09-16 16024s",
        "2024-09-17 7895s",
        "2024-09-19 24536s",
        "2024-09-20 203s",
        "2024-09-23 5478s",
        "2024-09-24 15247s",
        "2024-09-25 9134s",
        "2024-09-26 5724s",
        "2024-09-28 6751s",
        "2024-09-29 621s",
    ];

    let result = date_binary_search(&split_newline, &"2024-09-15".to_string());

    assert!(result.is_some());
}
