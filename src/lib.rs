//! # Rocket League Hours Tracker
//! This was made specifically for the Epic Games version of Rocket League
//! as the Epic Games launcher has no way of showing the past two hours played in
//! the same way that steam is able to.
//!
//! However, this program can and should still work with the steam version of the game.
//!
//! It is `HIGHLY` recommended to not manually alter the files that are created by this program
//! otherwise it could lead to unwanted behaviour by the program
//!
//! ``` rust
//!     println!("You got it Oneil :)");
//! ```

//! ## Library
//! The Rocket League Hours Tracker library contains modules which provide additional
//! functionality to the Rocket League Hours Tracker binary. This library currently
//! implements the [`website_files`] module, which provides the functionality to generate
//! the Html, CSS, and JavaScript for the Rocket League Hours Tracker website, and the [`update`]
//! module, which is the built in updater for the binary which retrieves the update from the GitHub
//! repository.
//!
//! The website functionality takes adavantage of the [`build_html`] library, which allows us
//! to generate the Html for the website, alongside the [`webbrowser`] library, which allows us
//! to open the website in a browser.
//!
//! The update module only operates when using the installed version of the program which can be found in the
//! [releases](https://github.com/OneilNvM/rl-hours-tracker/releases) section on the GitHub repository. This
//! module uses the [`reqwest`] crate to make HTTP requests to the rl-hours-tracker repository in order to retrieve
//! the new update from the releases section. This module has the functionality to check for any new updates, update
//! the program, and clean up any additional files made during the update.
//!
//! ### Use Case
//! Within the [`website_files`] module, there is a public function [`website_files::generate_website_files`],
//! which writes the files for the website in the website directory in `RlHoursFolder`. This function accepts a
//! [`bool`] value, which determines whether the option to open the website in a browser should appear when this
//! function is called.
//!
//! ```
//! use rl_hours_tracker::website_files;
//!
//! // This will generate the website files and prompt you with the option to open the
//! // webstie in a browser.
//! website_files::generate_website_files(true);
//!
//! // This will also generate the website but will not prompt the user to open the website
//! // in a browser.
//! website_files::generate_website_files(false);
//! ```
//!
//! The [`update`] module has two public asynchronous functions available: [`update::check_for_update`] and [`update::update`].
//! The [`update::check_for_update`] function is responsible for sending a HTTP request to the repository and checking the version
//! number of the latest release, and comparing it to the current version of the program. The [`update::update`] function is responsible
//! updating the program by sending a HTTP request to the repository to retrieve the update zip from the latest release, and unzipping the
//! zip files contents to replace the old program files with the newest version.
//!
//! ```
//! use rl_hours_tracker::update;
//! use tokio::runtime::Runtime;
//!
//! // This creates a tokio runtime instance for running our function
//! let rt = Runtime::new().unwrap();
//!
//! // This runs our asynchronous function which checks for an update
//! rt.block_on(update::check_for_update())?;
//! ```
//!
//! The [`update::check_for_update`] function does use the [`update::update`] function when it finds that there is a new release on the GitHub, however
//! the update function can be used by itself in a different context if needed.
//!
//! ```
//! use rl_hours_tracker::update;
//! use tokio::runtime::Runtime;
//!
//! // This creates a tokio runtime instance for running our function
//! let rt = Runtime::new().unwrap();
//!
//! // This runs our asynchronous function which updates the program
//! rt.block_on(update::update())?;
//! ```
use chrono::{prelude::*, Duration as CDuration};
use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::{self, Read, Write},
    process, thread,
    time::{Duration, SystemTime},
};
use stopwatch::Stopwatch;
use sysinfo::System;
use tokio::runtime::Runtime;

#[cfg(test)]
mod tests;
pub mod update;
pub mod website_files;

/// Type alias for Results which only return [`std::io::Error`] as its error variant.
pub type IoResult<T> = Result<T, io::Error>;

/// Custom error for [`calculate_past_two`] function
#[derive(Debug, Clone)]
pub struct PastTwoError;

impl Display for PastTwoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "next closest date to the date two weeks ago could not be found."
        )
    }
}

impl Error for PastTwoError {}

/// This runs the [`update::check_for_update`] function
pub fn run_self_update() -> Result<(), Box<dyn Error>> {
    let rt = Runtime::new().unwrap();

    rt.block_on(update::check_for_update())?;

    Ok(())
}

/// This function runs the program
pub fn run() {
    let process_name = "RocketLeague.exe";
    let mut is_waiting = false;
    let mut option = String::with_capacity(3);

    // Run the main loop
    run_main_loop(process_name, &mut is_waiting, &mut option);
}

/// This function creates the directories for the program. It creates a local [`Vec<Result>`]
/// which stores [`fs::create_dir`] results.
///
/// This function then returns a [`Vec<Result>`] which stores any errors that may have occurred
///
/// # Errors
/// This function stores an [`io::Error`] in the output Vector if there was any issue creating a folder.
pub fn create_directory() -> Vec<IoResult<()>> {
    // Create the folder directories for the program
    let folder = fs::create_dir("C:\\RLHoursFolder");
    let website_folder = fs::create_dir("C:\\RLHoursFolder\\website");
    let website_pages = fs::create_dir("C:\\RLHoursFolder\\website\\pages");
    let website_css = fs::create_dir("C:\\RLHoursFolder\\website\\css");
    let website_js = fs::create_dir("C:\\RLHoursFolder\\website\\js");
    let website_images = fs::create_dir("C:\\RLHoursFolder\\website\\images");

    // Store the folder results in Vector
    let folder_vec: Vec<IoResult<()>> = vec![
        folder,
        website_folder,
        website_pages,
        website_css,
        website_js,
        website_images,
    ];

    // Iterate through all the folder creations and filter for any errors
    let result: Vec<IoResult<()>> = folder_vec.into_iter().filter(|f| f.is_err()).collect();

    result
}

/// This function runs the main loop of the program. This checks if the `RocketLeague.exe` process is running and
/// runs the [`record_hours`] function if it is running, otherwise it will continue to wait for the process to start.
fn run_main_loop(process_name: &str, is_waiting: &mut bool, option: &mut String) {
    'main_loop: loop {
        // Check if the process is running
        if check_for_process(process_name) {
            record_hours(process_name);

            // Generate the website files
            website_files::generate_website_files(true).unwrap_or_else(|e| {
                eprintln!("error generating website files: {e}\nKind: {}", e.kind())
            });

            *is_waiting = false;

            println!("End program (y/n)?\n");
            io::stdin().read_line(option).unwrap();

            if option.trim() == "y" || option.trim() == "Y" {
                break 'main_loop;
            } else if option.trim() == "n" || option.trim() == "N" {
                *option = String::with_capacity(3);
                continue;
            } else {
                println!("Unexpected input! Ending program.");
                break 'main_loop;
            }
        } else {
            // Print 'Waiting for Rocket League to start...' only once by changing the value of is_waiting to true
            if !*is_waiting {
                print!("Waiting for Rocket League to start.\r");
                io::stdout()
                    .flush()
                    .expect("could not flush the output stream");
                thread::sleep(Duration::from_millis(500));
                print!("Waiting for Rocket League to start..\r");
                io::stdout()
                    .flush()
                    .expect("could not flush the output stream");
                thread::sleep(Duration::from_millis(500));
                print!("Waiting for Rocket League to start...\r");
                io::stdout()
                    .flush()
                    .expect("could not flush the output stream");
                thread::sleep(Duration::from_millis(500));
                print!("{}[2K\r", 27 as char);
                print!("Waiting for Rocket League to start\r");
                io::stdout()
                    .flush()
                    .expect("could not flush the output stream");
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

/// This function takes in a reference string `process_name: &str` and starts a stopwatch
/// which keeps track of the amount of seconds that pass whilst the process is running.
/// The stopwatch is ended and the File operations are run at the end of the process.
/// The date and elapsed time are stored in the `date.txt` file and the hours is stored in
/// `hours.txt`
fn record_hours(process_name: &str) {
    let mut sw = Stopwatch::start_new();

    let mut timer = SystemTime::now()
        .checked_add(Duration::from_millis(995))
        .unwrap();

    let mut seconds: u8 = 0;
    let mut minutes: u8 = 0;
    let mut hours: u16 = 0;

    println!("\nRocket League is running\n");

    // Loop checks for when the process has ended
    loop {
        let delay = timer.duration_since(SystemTime::now()).unwrap();

        // Check if current seconds are greater than or equal to 1 minute
        if seconds == 59 {
            seconds = 0;
            minutes += 1;

            // Check if current minutes are greater than or equal to 1 hour
            if minutes == 60 {
                minutes = 0;
                hours += 1;
            }
        } else {
            seconds += 1;
        }
        print!("{}[2K\r", 27 as char);

        // Print the output for the timer
        if hours < 10 && minutes < 10 && seconds < 10 {
            print!("Time Elapsed: 0{}:0{}:0{}\r", hours, minutes, seconds);
        } else if hours >= 10 {
            if minutes < 10 && seconds < 10 {
                print!("Time Elapsed: {}:0{}:0{}\r", hours, minutes, seconds);
            } else if minutes < 10 && seconds >= 10 {
                print!("Time Elapsed: {}:0{}:{}\r", hours, minutes, seconds);
            } else if minutes >= 10 && seconds < 10 {
                print!("Time Elapsed: {}:{}:0{}\r", hours, minutes, seconds);
            } else {
                print!("Time Elapsed: {}:{}:{}\r", hours, minutes, seconds);
            }
        } else if hours < 10 && minutes >= 10 && seconds < 10 {
            print!("Time Elapsed: 0{}:{}:0{}\r", hours, minutes, seconds);
        } else if hours < 10 && minutes < 10 && seconds >= 10 {
            print!("Time Elapsed: 0{}:0{}:{}\r", hours, minutes, seconds);
        } else {
            print!("Time Elapsed: 0{}:{}:{}\r", hours, minutes, seconds);
        }

        // Flush the output
        io::stdout().flush().expect("could not flush output stream");

        if !check_for_process(process_name) {
            // Stop the stopwatch
            sw.stop();

            println!("\n~~~ Record Hours: START ~~~\n");

            let seconds: u64 = sw.elapsed_ms() as u64 / 1000;
            let hours: f32 = (sw.elapsed_ms() as f32 / 1000_f32) / 3600_f32;

            let hours_result = File::open("C:\\RLHoursFolder\\hours.txt");
            let date_result = File::open("C:\\RLHoursFolder\\date.txt");

            // Write date and seconds to date.txt
            write_to_date(date_result, &seconds).unwrap_or_else(|e| {
                eprintln!("error writing to date.txt: {e}");
                process::exit(1);
            });

            // Buffer which stores the hours in the past two weeks
            let hours_buffer = calculate_past_two().unwrap_or_else(|e| {
                eprintln!("error calculating past two: {e}");
                0
            });

            if hours_buffer != 0 {
                let hours_past_two = hours_buffer as f32 / 3600_f32;

                write_to_hours(hours_result, &seconds, &hours, &hours_past_two, &sw)
                    .unwrap_or_else(|e| {
                        eprintln!("error writing to hours.txt: {e}");
                        process::exit(1);
                    });
                println!("\n~~~ Record Hours: FINISHED ~~~\n")
            } else {
                break;
            }

            break;
        }
        thread::sleep(delay);

        timer += Duration::from_millis(995)
    }
}

/// This function updates the hours in the past two weeks in the `hours.txt` file.
/// The hours past two is calculated through the [`calculate_past_two`] function
/// The function returns a [`Result<bool>`] if the function was able to successfully
/// update the hours past two, and write it to the `hours.txt` file.
///
/// # Errors
/// Returns an [`io::Error`] if there were any issues with file operations.
pub fn update_past_two() -> Result<bool, Box<dyn Error>> {
    let hours_file_result = File::open("C:\\RLHoursFolder\\hours.txt");

    // Buffer which stores the hours in the past two weeks
    let hours_buffer = calculate_past_two().unwrap_or_else(|e| {
        eprintln!("error calculating past two: {e}\n");
        0
    });

    // Check the value of the buffer
    let hours_past_two: f32 = if hours_buffer != 0 {
        hours_buffer as f32 / 3600_f32
    } else {
        return Ok(false);
    };

    // Checks if the 'hours.txt' file exists
    match hours_file_result {
        Ok(mut file) => {
            let mut hours_file_str = String::new();

            // Attempt to read from old hours.txt file
            match file.read_to_string(&mut hours_file_str) {
                Ok(_) => {
                    let (seconds, hours) = retrieve_time(&hours_file_str)?;

                    let write_hours_result = File::create("C:\\RLHoursFolder\\hours.txt");

                    // Attempt to write to the file
                    match write_hours_result {
                        Ok(mut w_file) => {
                            let rl_hours_str = format!("Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}hrs\nHours Past Two Weeks: {:.1}hrs\n", seconds, hours, hours_past_two);

                            // Check if the write succeeds
                            match w_file.write_all(rl_hours_str.as_bytes()) {
                                Ok(_) => {
                                    // Update website files
                                    website_files::generate_website_files(false).unwrap_or_else(
                                        |e| {
                                            eprintln!(
                                                "error generating website files: {e}\nKind: {}",
                                                e.kind()
                                            )
                                        },
                                    );
                                    Ok(true)
                                }
                                Err(e) => Err(Box::new(e)),
                            }
                        }
                        Err(e) => Err(Box::new(e)),
                    }
                }
                Err(e) => Err(Box::new(e)),
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

/// This function takes the `contents: &str` parameter which contains the contents from the `hours.txt` file
/// and returns a tuple of `(u64, f32)` which contains the seconds and hours from the file.
fn retrieve_time(contents: &str) -> Result<(u64, f32), Box<dyn Error>> {
    // Split the contents by newline character
    let split_new_line: Vec<&str> = contents.split("\n").collect();

    // Split the seconds and hours string references by whitspace
    let split_whitspace_sec: Vec<&str> = split_new_line[1].split_whitespace().collect();
    let split_whitespace_hrs: Vec<&str> = split_new_line[2].split_whitespace().collect();

    // Split the seconds and hours string references by characters
    let split_char_sec = split_whitspace_sec[2].chars();
    let split_char_hrs = split_whitespace_hrs[2].chars();

    let mut sec_vec: Vec<char> = vec![];
    let mut hrs_vec: Vec<char> = vec![];

    // Loop through Chars iterator to push only numeric characters to the seconds Vector
    for num in split_char_sec {
        if num.is_numeric() {
            sec_vec.push(num);
        }
    }

    // Loop through the Chars iterator to push numeric characters (plus the period character for decimals) to the hours Vector
    for num in split_char_hrs {
        if num.is_numeric() || num == '.' {
            hrs_vec.push(num);
        }
    }

    let seconds_str: String = sec_vec.iter().collect();
    let hours_str: String = hrs_vec.iter().collect();

    let old_seconds: u64 = seconds_str.parse()?;
    let old_hours: f32 = hours_str.parse()?;

    // Return a tuple of the old seconds and old hours
    Ok((old_seconds, old_hours))
}

/// This function takes a reference of a [`Vec<&str>`] Vector and returns a [`Some`] with the index of the closest
/// after the date two weeks ago.
pub fn closest_date(split_newline: &[&str]) -> Option<usize> {
    let today = Local::now().date_naive();
    let mut current_date = today - CDuration::days(14);

    // Find the closest date to the date two weeks ago
    while current_date <= today {
        let idx = date_binary_search(split_newline, &current_date.to_string());

        if let Some(index) = idx {
            return Some(index);
        }

        current_date += CDuration::days(1);
    }

    // Return None if the date is not found
    None
}

/// This function is used to perform a binary search on a [`Vec<&str>`] Vector and compares the dates in the Vector with
/// the `c_date` [`String`]. The function then returns a [`Some`] with the index of the date, or a [`None`] if the
/// date is not present.
/// 
/// ## Future Update - 08/12/2025
/// This function will be updated to be more optimal soon.
pub fn date_binary_search(split_newline: &[&str], c_date: &String) -> Option<usize> {
    let mut high = split_newline.len() - 1;
    let mut low = 0;
    let mut result = 0;
    let mut check_dups = false;
    let mut not_found = true;

    while low <= high {
        let mid = low + (high - low) / 2;

        let s_mid: Vec<&str> = split_newline[mid].split_whitespace().collect();

        if s_mid[0] == c_date {
            result = mid;
            not_found = false;
            check_dups = true;
            break;
        } else if *s_mid[0] < **c_date {
            low = mid + 1;
        } else {
            if mid == 0 {
                return None
            }
            high = mid - 1;
        }
    }

    while check_dups {
        let date_vec: Vec<&str> = split_newline[result].split_whitespace().collect();
        let date_str = date_vec[0];

        if result == 0 {
            check_dups = false;
            continue;
        }

        let ptr = result - 1;

        let prev_date_vec: Vec<&str> = split_newline[ptr].split_whitespace().collect();
        let prev_date_str = prev_date_vec[0];

        if date_str != prev_date_str {
            return Some(result);
        } else {
            result = ptr;
        }
    }

    if not_found {
        None
    } else {
        Some(result)
    }
}

/// This function calculates the hours recorded in the past two weeks and returns the total number of seconds as [`prim@u64`]
/// The contents from `date.txt` are read and split by `\n` character and stored in a [`Vec<&str>`] Vector.
/// It is then ordered and looped through in order to compare the date to the current iteration of the date two weeks ago.
/// The seconds are retrieved from the dates that match the current date in the iteration of the while loop and the seconds
/// are added to `seconds_past_two` which is returned as an [`Result<u64>`] at the end of the function.
///
/// # Errors
/// This function returns [`Box<dyn Error>`] which could potentially be two types of errors:
/// - A [`PastTwoError`], which is a custom error which occurs when [`closest_date`] fails.
/// - An [`io::Error`], which occurs when the `date.txt` file could not be opened, or read.
pub fn calculate_past_two() -> Result<u64, Box<dyn Error>> {
    let date_file_result = File::open("C:\\RLHoursFolder\\date.txt");
    let mut seconds_past_two: u64 = 0;

    // Check if the date.txt file exists
    match date_file_result {
        Ok(mut date_file) => {
            println!("\n~~~ Calculate Past Two: START ~~~\n");
            let mut date_file_str = String::new();

            // Checks if the file was read successfully
            match date_file.read_to_string(&mut date_file_str) {
                Ok(_) => {
                    println!("Dates retrieved...");
                    let mut split_newline: Vec<&str> = date_file_str.split("\n").collect();
                    split_newline.pop();
                    split_newline.sort();

                    let today = Local::now().date_naive();
                    let mut is_after_today = false;
                    let two_weeks_ago = today - CDuration::days(14);
                    let mut cur_date: NaiveDate = two_weeks_ago;

                    // Declare variable for string reference slice
                    let split_line_copy: &[&str];

                    println!("Finding date two weeks ago...");
                    let date_idx = date_binary_search(&split_newline, &cur_date.to_string());

                    // Checks the value returned from date_binary_search
                    match date_idx {
                        Some(index) => {
                            println!("Date found...");
                            split_line_copy = &split_newline[index..];
                        }
                        // Find closest date if the target date was not found
                        None => {
                            println!("Date not found. Searching for closest date...");
                            let closest = closest_date(&split_newline);

                            match closest {
                                Some(index) => {
                                    println!("Date found...");
                                    split_line_copy = &split_newline[index..];
                                }
                                None => return Err(PastTwoError.into()),
                            }
                        }
                    }

                    println!("Calculating past two...");
                    while !is_after_today {
                        // Checks if the current iteration of the date two weeks ago is greater than today
                        if cur_date > today {
                            is_after_today = true;
                            continue;
                        }

                        for date in split_line_copy {
                            let split_whitespace: Vec<&str> = date.split_whitespace().collect();

                            // Check if cur_date is equivalent to the date from split_whitespace Vector
                            if cur_date.to_string() == split_whitespace[0] {
                                let split_chars = split_whitespace[1].chars();

                                let mut sec_vec: Vec<char> = vec![];

                                for num in split_chars {
                                    if num.is_numeric() {
                                        sec_vec.push(num);
                                    }
                                }

                                let seconds_str: String = sec_vec.iter().collect();
                                let total_seconds: u64 = seconds_str.parse().unwrap();

                                // Add the total seconds to the seconds_past_two variable
                                seconds_past_two += total_seconds;
                            }
                        }

                        // Increase the current date to the next day
                        cur_date += CDuration::days(1);
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        Err(e) => return Err(e.into()),
    }

    println!("Past two calculated\n\n~~~ Calculate Past Two: FINISHED ~~~\n");
    Ok(seconds_past_two)
}

/// This function constructs a new [`String`] which will have the contents to write to `hours.txt` with new hours and seconds
/// and returns it.
fn return_new_hours(contents: &str, seconds: &u64, hours: &f32, past_two: &f32) -> Result<String, Box<dyn Error>> {
    println!("Getting old hours...");
    // Retrieves the old hours and seconds from the contents String
    let (old_seconds, old_hours) = retrieve_time(contents)?;

    let added_seconds = old_seconds + *seconds;
    let added_hours = old_hours + *hours;

    Ok(format!(
        "Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}hrs\nHours Past Two Weeks: {:.1}hrs\n",
        added_seconds, added_hours, past_two
    ))
}

/// This function writes the new contents to the `hours.txt` file. This includes the total `seconds`, `hours`, and `hours_past_two`.
/// This function then returns a [`Result<()>`] when file operations were all successful.
///
/// # Errors
/// This function returns an [`io::Error`] if any file operations failed.
fn write_to_hours(
    hours_result: IoResult<File>,
    seconds: &u64,
    hours: &f32,
    hours_past_two: &f32,
    sw: &Stopwatch,
) -> Result<(), Box<dyn Error>> {
    // Check if the file exists
    if let Ok(mut file) = hours_result {
        let mut contents = String::new();

        // Attempt to read from the hours.txt file
        match file.read_to_string(&mut contents) {
            Ok(_) => {
                // Stores the new contents for the file as a String
                let rl_hours_str = return_new_hours(&contents, seconds, hours, hours_past_two)?;

                let truncated_file = File::create("C:\\RLHoursFolder\\hours.txt");

                // Attempt to write to hours.txt
                match truncated_file {
                    Ok(mut t_file) => {
                        println!("Writing to hours.txt...");
                        // Check if the write was successful
                        match t_file.write_all(rl_hours_str.as_bytes()) {
                            Ok(_) => {
                                println!("Successful!");
                                Ok(())
                            }
                            Err(e) => Err(Box::new(e)),
                        }
                    }
                    Err(e) => Err(Box::new(e)),
                }
            }
            Err(e) => Err(Box::new(e)),
        }
    } else {
        // Check if the file was created successfully
        match File::create("C:\\RLHoursFolder\\hours.txt") {
            Ok(mut file) => {
                let total_seconds = sw.elapsed_ms() / 1000;
                let total_hours: f32 = (sw.elapsed_ms() as f32 / 1000_f32) / 3600_f32;
                let rl_hours_str = format!(
                                "Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}hrs\nHours Past Two Weeks: {:.1}hrs\n", total_seconds, total_hours, hours_past_two
                            );

                println!("Writing to hours.txt...");
                // Checks if the write was successful
                match file.write_all(rl_hours_str.as_bytes()) {
                    Ok(_) => {
                        println!("The hours file was successfully created");
                        Ok(())
                    }
                    Err(e) => Err(Box::new(e)),
                }
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}

/// This function writes new contents to the `date.txt` file. This uses the [`Local`] struct which allows us to use the [`Local::now()`]
/// function to retrieve the local date and time as [`DateTime<Local>`]. The date is then turned into a [`NaiveDate`] by using [`DateTime<Local>::date_naive()`]
/// which returns us the date by itself.
///
/// # Errors
/// Returns an [`io::Error`] if there were any file operations which failed.
fn write_to_date(date_result: IoResult<File>, seconds: &u64) -> IoResult<()> {
    // Check if the date file exists
    if date_result.is_ok() {
        let append_date_result = File::options()
            .append(true)
            .open("C:\\RLHoursFolder\\date.txt");

        // Attenot to open the date.txt file
        match append_date_result {
            Ok(mut date_file) => {
                let today = Local::now().date_naive();

                let today_str = format!("{} {}s\n", today, seconds);

                println!("Appending to date.txt...");
                // Checks if the write was successful
                match date_file.write_all(today_str.as_bytes()) {
                    Ok(_) => {
                        println!("Successful!");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    } else {
        // Check if the file was created
        match File::create("C:\\RLHoursFolder\\date.txt") {
            Ok(mut file) => {
                let today = Local::now().date_naive();

                let today_str = format!("{} {}s\n", today, seconds);

                println!("Appending to date.txt...");
                // Checks if the write was successful
                match file.write_all(today_str.as_bytes()) {
                    Ok(_) => {
                        println!("The date file was successfully created");
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// This function checks if the process passed in via `name: &str` is running and returns a [`bool`] value
fn check_for_process(name: &str) -> bool {
    let sys = System::new_all();
    let mut result = false;

    for process in sys.processes_by_exact_name(name.as_ref()) {
        if process.name() == name {
            result = true;
            break;
        }
    }

    result
}
