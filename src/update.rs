//! Check for updates on the private repo
//!
//! Private repo hosts the zip file with new binary
//!
//! If there is a new release, send get request to temporarily download zip file
//!
//! Extract the exe file from zip file
//!
//! Update the old exe file in the Program Files location with the new version
//!
//! Delete the temporary zip file and extracts
//!
//! Finish up anything else
use reqwest::{
    self,
    Client
};
use std::{fs, io};

pub async fn check_for_update() {
    let client = Client::new();

    let response = client
        .get("https://github.com/OneilNvM/rl-hours-tracker/releases/latest")
        .send()
        .await
        .unwrap();

    let url = response.url().to_string();

    let url_vec: Vec<&str> = url.rsplit("/").collect();

    let version = url_vec[0].replace("v", "");

    if version == env!("CARGO_PKG_VERSION") {
        println!("Latest Version: {version}")
    } else {
        let mut option = String::new();

        println!("NEW VERSION AVAILABLE!!\n\nUpdate to version '{version}' (y/n)?");

        io::stdin().read_line(&mut option).unwrap();

        if option.trim().to_lowercase() == "y" {
            println!("\nDownloading update...\n");
            test_update().await;
        } else {
            ()
        }
    }
}


async fn test_update() {
    let client = Client::new();

    let response = client
    .get("https://github.com/OneilNvM/rl-hours-tracker/releases/download/v0.3.5/rlht-setup.exe")
    .send()
    .await
    .unwrap();

    let download = response.bytes().await.unwrap();

    fs::write("C:\\Program Files\\Rocket League Hours Tracker\\rlht-setup.exe", download).unwrap();

    println!("Download complete!\n");
}
