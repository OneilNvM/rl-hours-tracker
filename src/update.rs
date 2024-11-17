//! This module is responsible for performing update operations for the Rocket League Hours Tracker binary,
//! which can be installed through the GitHub repository [releases](https://github.com/OneilNvM/rl-hours-tracker/releases)
//! section.
use bytes::Bytes;
use core::str;
use directories::BaseDirs;
use reqwest::{self, Client};
use std::{env, error::Error, fs, io, path::PathBuf, process, thread, time::Duration};
use zip;

/// Asynchronous function which checks the the GitHub repository for the latest release
/// of the program.
///
/// If there is a new release, the function then runs the [`update`] function to replace the
/// old files for the program with the new files from the `update.zip` archive on github.
///
/// # Errors
/// This function returns a [`reqwest::Error`] if there were any errors sending `GET` request to GitHub
/// or any error from the [`update`] function.
pub async fn check_for_update() -> Result<(), Box<dyn Error>> {
    // Check if there was a prior update to finish any additional cleanup
    let get_prior_update = process::Command::new("cmd")
        .args(["/C", "set PRIOR_UPDATE"])
        .output();

    match get_prior_update {
        Ok(output) => {
            let output_string = str::from_utf8(&output.stdout).unwrap();

            if output_string.contains("1") {
                additional_cleanup()?
            }
        }
        Err(e) => {
            eprintln!("issue getting PRIOR_UPDATE: {e}");
        }
    }

    // Create a new Client instance
    let client = Client::new();

    // Send a GET request to the GitHub for the latest release
    let response = client
        .get("https://github.com/OneilNvM/rl-hours-tracker/releases/latest")
        .send()
        .await?;

    // Return the final url as a String
    let url = response.url().to_string();

    // Store a reverse split vector of the url separated by '/' character
    let url_vec: Vec<&str> = url.rsplit("/").collect();

    // Get the version number
    let version = url_vec[0].replace("v", "");

    // Check if the latest version is equal to the current version
    if version == env!("CARGO_PKG_VERSION") {
        println!("Latest Version: {version}");
        Ok(())
    } else {
        let mut option = String::new();

        println!("NEW VERSION AVAILABLE!!\n\nUpdate to version '{version}' (y/n)?");

        io::stdin().read_line(&mut option).unwrap();

        // Check if the user wants to update or not
        if option.trim().to_lowercase() == "y" {
            println!("\nDownloading update...\n");
            update(&version).await?;
            Ok(())
        } else {
            Ok(())
        }
    }
}

/// This function updates the Rocket League Hours Tracker binary.
///
/// A HTTP `GET` request is sent to the GitHub repo's release section to download the bytes
/// for `update.zip`.
/// The zip is then extracted and the new files replace the old files.
///
/// # Errors
/// This function returns file operation errors or a [`reqwest::Error`].
pub async fn update(ver_num: &str) -> Result<(), Box<dyn Error>> {
    // Create a new Client instance
    let client = Client::new();

    // Store the url to download the zip file from
    let url = format!(
        "https://github.com/OneilNvM/rl-hours-tracker/releases/download/v{ver_num}/update.zip"
    );

    // Send the GET request to the GitHub repository for the 'update.zip' archive
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        println!("The newest update includes changes to the built-in updater.");
        thread::sleep(Duration::from_secs(3));
        println!("You will need to download the newest installer from GitHub.");
        thread::sleep(Duration::from_secs(5));
        process::exit(0)
    }

    // Return the Bytes of the zip archive
    let download = response.bytes().await?;

    // Store the application's directory
    let base_dir = BaseDirs::new().unwrap();
    let app_dir = base_dir
        .config_local_dir()
        .join("Programs")
        .join("Rocket League Hours Tracker");

    // Create the tmp folder for the zip archive
    let tmp_result = fs::create_dir(app_dir.join("tmp"));

    // Handle the Result returned by the 'tmp_result' variable
    if tmp_result.is_err() {
        eprintln!("error creating tmp directory.\ncreating zip file locally.\n");
        extract_local_zip(&app_dir, &download)?;
    } else {
        extract_update(app_dir, download)?;
    }

    println!("Update complete!\n");
    thread::sleep(Duration::from_millis(1000));
    println!("Please wait for the program to close...");
    thread::sleep(Duration::from_millis(5000));

    // Set the 'PRIOR_UPDATE' environment variable
    let set_prior_update = process::Command::new("cmd")
        .args(["/C", "setx", "PRIOR_UPDATE", "1"])
        .status();

    if let Err(e) = set_prior_update {
        eprintln!("issue setting up PRIOR_UPDATE: {e}");
    }

    process::exit(0)
}

fn additional_cleanup() -> Result<(), io::Error> {
    let base_dir = BaseDirs::new().unwrap();
    let app_dir = base_dir
        .config_local_dir()
        .join("Programs")
        .join("Rocket League Hours Tracker");

    fs::remove_file(app_dir.join("old-rl-hours-tracker.exe"))?;

    let change_prior_update = process::Command::new("cmd")
        .args(["/C", "setx", "PRIOR_UPDATE", "0"])
        .status();

    if let Err(e) = change_prior_update {
        eprintln!("issue changing PRIOR_UPDATE: {e}");
    }

    println!("Cleanup successful");

    Ok(())
}

fn extract_update(app_dir: PathBuf, download: Bytes) -> Result<(), Box<dyn Error>> {
    println!("Created 'tmp' directory...");

    let file_name = app_dir.join("tmp").join("update.zip");

    fs::write(file_name, download)?;

    println!("Downloaded 'update.zip' archive...");

    fs::rename(
        app_dir.join("rl-hours-tracker.exe"),
        app_dir.join("old-rl-hours-tracker.exe"),
    )?;

    println!("Removing old files...");

    fs::remove_file(app_dir.join("unins000.dat"))?;
    fs::remove_file(app_dir.join("unins000.exe"))?;

    let update = fs::File::open(app_dir.join("tmp\\update.zip"))?;

    println!("Extracting update files...");

    let mut archive = zip::ZipArchive::new(update)?;
    archive.extract(&app_dir)?;

    println!("Update files extracted");

    fs::remove_dir_all(app_dir.join("tmp"))?;

    println!("Removed tmp directory...\n");

    Ok(())
}

fn extract_local_zip(app_dir: &PathBuf, download: &Bytes) -> Result<(), Box<dyn Error>> {
    let file_name = app_dir.join("update.zip");

    fs::write(file_name, download)?;

    println!("Downloaded 'update.zip' archive locally...");

    fs::rename(
        app_dir.join("rl-hours-tracker.exe"),
        app_dir.join("old-rl-hours-tracker.exe"),
    )?;

    fs::remove_file(app_dir.join("unins000.dat"))?;
    fs::remove_file(app_dir.join("unins000.exe"))?;

    println!("Removing old files...");

    let update = fs::File::open(app_dir.join("update.zip"))?;

    println!("Extracting update files...");

    let mut archive = zip::ZipArchive::new(update)?;
    archive.extract(app_dir)?;

    println!("Update files extracted");

    fs::remove_file(app_dir.join("update.zip"))?;

    println!("Removed 'update.zip' archive...\n");

    Ok(())
}
