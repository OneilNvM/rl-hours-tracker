use crate::website_files::*;

#[test]
fn test_sets_github_fields() {
    let mut instance = Github::new();

    instance.set_fields(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/images",
        "rl-icon-black.png",
    );

    let fields_vec: Vec<&str> = vec![
        instance.owner,
        instance.repo,
        instance.branch,
        instance.path,
        instance.file,
    ];

    assert_eq!(
        fields_vec,
        vec![
            "OneilNvM",
            "rl-hours-tracker",
            "master",
            "website/images",
            "rl-icon-black.png"
        ]
    );
}

#[test]
fn test_builds_raw_url() {
    let mut instance = Github::new();

    instance.set_fields(
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
fn test_builds_image_url() {
    let mut instance = Github::new();

    instance.set_fields(
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
async fn test_sends_request() -> Result<(), reqwest::Error> {
    let mut instance = Github::new();
    instance.set_fields(
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