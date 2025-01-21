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
    // String reference of the Rocket League process name
    let process_name = "RocketLeague.exe";
    // Mutable boolean to determine when the program is waiting for the process to run
    let mut is_waiting = false;
    // Mutable string for user option
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
    // Main loop for the program
    'main_loop: loop {
        // Checks if the process is running
        if check_for_process(process_name) {
            // Begins the loop which records the seconds past after the process began
            record_hours(process_name);

            // Generate the website files
            website_files::generate_website_files(true).unwrap_or_else(|e| {
                eprintln!("error generating website files: {e}\nKind: {}", e.kind())
            });

            // Change is_waiting value back to false
            *is_waiting = false;

            // Allow user to choose whether to continue the program or end it
            println!("End program (y/n)?\n");
            io::stdin().read_line(option).unwrap();

            // Check the option the user gave and respond accordingly
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
    // Start the stopwatch
    let mut sw = Stopwatch::start_new();

    // Create a SystemTime struct with the time set to 995ms in the future
    // This is done to make the timing for seconds more accurate when sleeping on each iteration
    // by predicting the time in the future
    let mut timer = SystemTime::now()
        .checked_add(Duration::from_millis(995))
        .unwrap();

    // Declare mutable variables for seconds, minutes and hours
    let mut seconds: u8 = 0;
    let mut minutes: u8 = 0;
    let mut hours: u32 = 0;

    println!("\nRocket League is running\n");

    // Loop checks for when the process has ended
    loop {
        // Set the delay for the for how long the loop sleeps
        let delay = timer.duration_since(SystemTime::now()).unwrap();
        // This control flow is used for displaying the live stopwatch
        // whilst Rocket League is running.
        // The loop runs on a 1 second sleep at every iteration, which allows
        // for the live stopwatch to update, displaying the current time elapsed.

        // Check if current seconds are greater than or equal to 1 minute
        if seconds == 59 {
            // Set seconds back to zero and update the minutes by 1
            seconds = 0;
            minutes += 1;

            // Checks if current minutes are greater than or equal to 1 hour
            if minutes == 60 {
                // Set minutes back to zero and update hours by 1
                minutes = 0;
                hours += 1;
            }
        } else {
            // Increment the seconds by 1
            seconds += 1;
        }
        // Clear the current line and carriage return
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
            // Stops the stopwatch
            sw.stop();

            println!("\n~~~ Record Hours: START ~~~\n");

            // Stores the seconds elapsed as u64
            let seconds: u64 = sw.elapsed_ms() as u64 / 1000;
            // Stores the hours as f32
            let hours: f32 = (sw.elapsed_ms() as f32 / 1000_f32) / 3600_f32;

            // Opens both the hours file and date file in read mode if they exist
            let hours_result = File::open("C:\\RLHoursFolder\\hours.txt");
            let date_result = File::open("C:\\RLHoursFolder\\date.txt");

            // Calls the function which writes the date the program is run, along with the seconds elapsed during
            // the session to the 'date.txt' file
            write_to_date(date_result, &seconds).unwrap_or_else(|e| {
                eprintln!("error writing to date.txt: {e}");
                process::exit(1);
            });

            // Buffer which stores the hours in the past two weeks
            // The 'Some' value is unwrapped if there were no issues or 'u64::MAX'
            // is the value if there were issues
            let hours_buffer = calculate_past_two().unwrap_or_else(|e| {
                eprintln!("error calculating past two: {e}");
                0
            });

            // This condition checks the value of the buffer
            if hours_buffer != 0 {
                // Stores the hours in the past two weeks as f32
                let hours_past_two = hours_buffer as f32 / 3600_f32;

                // Calls the function to write the total seconds, hours, and hours in the past two weeks
                // to the 'hours.txt' file
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
        // Sleep for as long as the delay
        thread::sleep(delay);

        // Update the timer to 995ms in the future
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
    // Open the 'hours.txt' file in read mode
    let hours_file_result = File::open("C:\\RLHoursFolder\\hours.txt");

    // Buffer which stores the hours in the past two weeks
    // The 'Some' value is unwrapped if there were no issues or 'u64::MAX'
    // is the value if there were issues
    let hours_buffer = calculate_past_two().unwrap_or_else(|e| {
        eprintln!("error calculating past two: {e}\n");
        0
    });

    // This condition checks the value of the buffer
    let hours_past_two: f32 = if hours_buffer != 0 {
        // Set hours_past_two variable to the buffer value
        hours_buffer as f32 / 3600_f32
    } else {
        // Returns false
        return Ok(false);
    };

    // Checks if the 'hours.txt' file exists, then stores the File in the mutable 'file' variable
    match hours_file_result {
        Ok(mut file) => {
            // Create a new empty String
            let mut hours_file_str = String::new();

            // Match statement returns the true if operation was successful or panics if there was an error
            match file.read_to_string(&mut hours_file_str) {
                Ok(_) => {
                    // Deconstruct the seconds and hours tuple returned by the retrieve_time function
                    let (seconds, hours) = retrieve_time(&hours_file_str)?;

                    // Creates or truncates the 'hours.txt' file and opens it in write mode
                    let write_hours_result = File::create("C:\\RLHoursFolder\\hours.txt");

                    // Checks if the 'hours.txt' file was created, then stores the File in the mutable 'w_file' variable
                    match write_hours_result {
                        Ok(mut w_file) => {
                            // Stores the new contents of the file as String
                            let rl_hours_str = format!("Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}hrs\nHours Past Two Weeks: {:.1}hrs\n", seconds, hours, hours_past_two);

                            // Checks if writing to the file was successful
                            match w_file.write_all(rl_hours_str.as_bytes()) {
                                // Update the website files and returns true
                                Ok(_) => {
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
                                // Returns an error if there was an issue when writing to the file
                                Err(e) => Err(Box::new(e)),
                            }
                        }
                        // Returns an error if there was an issue creating the file
                        Err(e) => Err(Box::new(e)),
                    }
                }
                // Returns an error if there was an issue reading the file
                Err(e) => Err(Box::new(e)),
            }
        }
        // Returns an error if there was an issue opening the file
        Err(e) => Err(Box::new(e)),
    }
}

/// This function takes the `contents: &String` parameter which contains the contents from the `hours.txt` file
/// and returns a tuple of `(u64, f32)` which contains the seconds and hours from the file.
fn retrieve_time(contents: &str) -> Result<(u64, f32), Box<dyn Error>> {
    // Split the contents string down until we have the characters we want from the string
    // Specifically, we want the seconds and hours numbers from the file
    // First split the contents by newline character
    let split_new_line: Vec<&str> = contents.split("\n").collect();

    // Split the seconds and hours string references by whitspace
    let split_whitspace_sec: Vec<&str> = split_new_line[1].split_whitespace().collect();
    let split_whitespace_hrs: Vec<&str> = split_new_line[2].split_whitespace().collect();

    // Split the seconds and hours string references by characters
    let split_char_sec = split_whitspace_sec[2].chars();
    let split_char_hrs = split_whitespace_hrs[2].chars();

    // Declare and initialize Vector with type char
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

    // Collect the Vector characters as a String
    let seconds_str: String = sec_vec.iter().collect();
    let hours_str: String = hrs_vec.iter().collect();

    // Parse the seconds string as u64 and hours string as f32
    let old_seconds: u64 = seconds_str.parse()?;
    let old_hours: f32 = hours_str.parse()?;

    // Return a tuple of the old seconds and old hours
    Ok((old_seconds, old_hours))
}

/// This function takes a reference of a [`Vec<&str>`] Vector and returns a [`Some`] with the index of the closest
/// after the date two weeks ago.
pub fn closest_date(split_newline: &[&str]) -> Option<usize> {
    // Store the local date today
    let today = Local::now().date_naive();
    // Store the date two weeks ago
    let mut current_date = today - CDuration::days(14);

    // While loop attempts to find what date is closest to the date two weeks ago, within the Vector
    while current_date <= today {
        // date_binary_search takes a reference of split_newline Vector and the current iteration of the date
        let idx = date_binary_search(split_newline, &current_date.to_string());

        // Returns the index of the closest date if value is not usize::MAX
        if let Some(index) = idx {
            return Some(index);
        }

        // Increments the date
        current_date += CDuration::days(1);
    }

    // Return None if the date is not found
    None
}

/// This function is used to perform a binary search on a [`Vec<&str>`] Vector and compares the dates in the Vector with
/// the `c_date` [`String`]. The function then returns a [`Some`] with the index of the date, or a [`None`] if the
/// date is not present.
pub fn date_binary_search(split_newline: &[&str], c_date: &String) -> Option<usize> {
    // Initialize mutable variables
    let mut high = split_newline.len() - 1;
    let mut low = 0;
    let mut result = 0;
    let mut check_dups = false;
    let mut not_found = true;

    // While loop performs binary search of the date in the Vector
    // Loop is broken if the index of the date is found
    while low <= high {
        // Set the midpoint of the current iteration
        let mid = low + (high - low) / 2;

        // Split the Vector element by whitespace
        let s_mid: Vec<&str> = split_newline[mid].split_whitespace().collect();

        // If statement checks if the current date is either equal, less than, or greater than
        // the date two weeks ago
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

    // While loop checks for any duplicates of the date two weeks ago in order to include them in the new Vector
    // This loop only runs if the date is found in the Vector
    while check_dups {
        // Date Vector for current iteration
        let date_vec: Vec<&str> = split_newline[result].split_whitespace().collect();
        // Date string reference for the current iteration
        let date_str = date_vec[0];

        // Check if result is equal to zero
        // End loop if true
        if result == 0 {
            check_dups = false;
            continue;
        }

        // Set the ptr to current iteration - 1
        let ptr = result - 1;

        // Initialize similar variables to the current iteration but with the pointer
        let prev_date_vec: Vec<&str> = split_newline[ptr].split_whitespace().collect();
        let prev_date_str = prev_date_vec[0];

        // Checks if the current iteration date is not equal to the past date
        // Return the index if true
        // Set the current iteration to the pointer if false
        if date_str != prev_date_str {
            return Some(result);
        } else {
            result = ptr;
        }
    }

    // Return None if the date is not found
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
    // Open the 'date.txt' file in read mode
    let date_file_result = File::open("C:\\RLHoursFolder\\date.txt");
    // Initialize a mutable variable as u64 for the seconds past two
    let mut seconds_past_two: u64 = 0;

    // Checks if the 'date.txt' file was opened, then stores File in the mutable 'date_file' variable
    match date_file_result {
        Ok(mut date_file) => {
            println!("\n~~~ Calculate Past Two: START ~~~\n");
            // Creates and empty String
            let mut date_file_str = String::new();

            // Checks if the file was read successfully
            match date_file.read_to_string(&mut date_file_str) {
                Ok(_) => {
                    println!("Dates retrieved...");
                    // Split the contents of file by newline character
                    let mut split_newline: Vec<&str> = date_file_str.split("\n").collect();

                    // Pops the end of the Vector as it is an empty string
                    split_newline.pop();

                    // Sorts the dates and puts them in order
                    split_newline.sort();

                    // Store the current local date
                    let today = Local::now().date_naive();

                    // Initialize a variable to keep track of when the date two weeks ago surpasses the current date
                    let mut is_after_today = false;

                    // Store the date two weeks ago
                    let two_weeks_ago = today - CDuration::days(14);

                    // Mutable variable of date two weeks ago
                    let mut cur_date: NaiveDate = two_weeks_ago;

                    // Declare variable for string reference slice
                    let split_line_copy: &[&str];

                    println!("Finding date two weeks ago...");
                    // Assign value from date_binary_search to variable
                    let date_idx = date_binary_search(&split_newline, &cur_date.to_string());

                    // Checks the value returned from date_binary_search
                    // Either sets the split_line_copy variable to a string reference slice of the Vector, with the first element
                    // as the first occurrence of the date two weeks ago
                    // Or, sets it to the closest date after the date two weeks ago
                    match date_idx {
                        Some(index) => {
                            println!("Date found...");
                            split_line_copy = &split_newline[index..];
                        }
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
                    // While loop checks if the date is in the contents string and adds the seconds accompanied with it, to the seconds_past_two variable
                    while !is_after_today {
                        // Checks if the current iteration of the date two weeks ago is greater than today
                        if cur_date > today {
                            is_after_today = true;
                            continue;
                        }

                        // Loop through split_line_copy vector and compare the date to the cur_date
                        for date in split_line_copy {
                            // Split the date by whitespace
                            let split_whitespace: Vec<&str> = date.split_whitespace().collect();

                            // Check if cur_date is equivalent to the date from split_whitespace Vector
                            if cur_date.to_string() == split_whitespace[0] {
                                // Split the seconds into characters
                                let split_chars = split_whitespace[1].chars();

                                // Initialize an empty Vector of type char
                                let mut sec_vec: Vec<char> = vec![];

                                // Loop through the split_chars variable and push only numerics to the Vector
                                for num in split_chars {
                                    if num.is_numeric() {
                                        sec_vec.push(num);
                                    }
                                }

                                // Collect the characters as a String
                                let seconds_str: String = sec_vec.iter().collect();
                                // Parse the String to a u64
                                let total_seconds: u64 = seconds_str.parse().unwrap();

                                // Add the total seconds to the seconds_past_two variable
                                seconds_past_two += total_seconds;
                            }
                        }

                        // Increase the current date to the next day
                        cur_date += CDuration::days(1);
                    }
                }
                // Returns an error if there was an issue reading the file
                Err(e) => return Err(e.into()),
            }
        }
        // Returns an error if there was an issue opening the file
        Err(e) => return Err(e.into()),
    }

    println!("Past two calculated\n\n~~~ Calculate Past Two: FINISHED ~~~\n");
    Ok(seconds_past_two)
}

/// This function constructs a new [`String`] which will have the contents to write to `hours.txt` with new hours and seconds
/// and returns it.
fn return_new_hours(contents: &str, seconds: &u64, hours: &f32, past_two: &f32) -> Result<String, Box<dyn Error>> {
    println!("Getting old hours...");
    // Retrieves the old time and seconds from the contents String
    let time = retrieve_time(contents)?;

    // Deconstruct the seconds and hours from the tuple
    let (old_seconds, old_hours) = time;

    // Add the new seconds and the new hours to the old
    let added_seconds = old_seconds + *seconds;
    let added_hours = old_hours + *hours;

    // Return the new string of the file contents to be written to the file
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
    // Checks if the file exists, then stores the File into the mutable 'file' variable
    if let Ok(mut file) = hours_result {
        // Mutable variable to store file contents as a String
        let mut contents = String::new();

        // Checks if the file reads successfully, write new data to the file
        match file.read_to_string(&mut contents) {
            Ok(_) => {
                // Stores the new contents for the file as a String
                let rl_hours_str = return_new_hours(&contents, seconds, hours, hours_past_two)?;

                // Opens the file in write mode
                let truncated_file = File::create("C:\\RLHoursFolder\\hours.txt");

                // Checks if the file was exists, then stores the truncated File into the mutable 't_file' variable
                match truncated_file {
                    Ok(mut t_file) => {
                        println!("Writing to hours.txt...");
                        // Checks if writing to the file was successful
                        match t_file.write_all(rl_hours_str.as_bytes()) {
                            Ok(_) => {
                                println!("Successful!");
                                Ok(())
                            }
                            // Returns an error if there was an issue writing to the file
                            Err(e) => Err(Box::new(e)),
                        }
                    }
                    // Returns an error if there was an issue creating the file
                    Err(e) => Err(Box::new(e)),
                }
            }
            // Returns an error if there was an issue reading the file
            Err(e) => Err(Box::new(e)),
        }
    } else {
        // Checks if the file was created successfully, then stores the File in the mutable 'file' variable
        match File::create("C:\\RLHoursFolder\\hours.txt") {
            Ok(mut file) => {
                // Store the total seconds, hours and the new String for the file in variables
                let total_seconds = sw.elapsed_ms() / 1000;
                let total_hours: f32 = (sw.elapsed_ms() as f32 / 1000_f32) / 3600_f32;
                let rl_hours_str = format!(
                                "Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}hrs\nHours Past Two Weeks: {:.1}hrs\n", total_seconds, total_hours, hours_past_two
                            );

                println!("Writing to hours.txt...");
                // Checks if writing to the file was successful
                match file.write_all(rl_hours_str.as_bytes()) {
                    Ok(_) => {
                        println!("The hours file was successfully created");
                        Ok(())
                    }
                    // Returns an error if there was any kind of issue during writing process
                    Err(e) => Err(Box::new(e)),
                }
            }
            // Returns an error if there was an issue when attempting to create the file
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
    // Checks if the date file exists, then handles file operations
    if date_result.is_ok() {
        // Opens the date file in append mode
        let append_date_result = File::options()
            .append(true)
            .open("C:\\RLHoursFolder\\date.txt");

        // Checks if the file opens, then stores the File in the mutable 'date_file' variable
        match append_date_result {
            Ok(mut date_file) => {
                // Store the current local date
                let today = Local::now().date_naive();

                // This String stores the date today, and the seconds elapsed in session
                let today_str = format!("{} {}s\n", today, seconds);

                println!("Appending to date.txt...");
                // Checks if writing to the file was successful
                match date_file.write_all(today_str.as_bytes()) {
                    Ok(_) => {
                        println!("Successful!");
                        Ok(())
                    }
                    // Returns an error if there was an issue writing to the file
                    Err(e) => Err(e),
                }
            }
            // Returns an error if there was an issue opening the file
            Err(e) => Err(e),
        }
    } else {
        // Checks if the file was created, then stores the File in the mutable 'file' variable
        match File::create("C:\\RLHoursFolder\\date.txt") {
            Ok(mut file) => {
                // Store the current local date
                let today = Local::now().date_naive();

                // This String stores the date today, and the seconds elapsed in session
                let today_str = format!("{} {}s\n", today, seconds);

                println!("Appending to date.txt...");
                // Checks if writing to the file was successful
                match file.write_all(today_str.as_bytes()) {
                    Ok(_) => {
                        println!("The date file was successfully created");
                        Ok(())
                    }
                    // Returns an error if there was an issue writing to the file
                    Err(e) => Err(e),
                }
            }
            // Returns an error if there was an issue creating the file
            Err(e) => Err(e),
        }
    }
}

/// This function checks if the process passed in via `name: &str` is running and returns a [`bool`] value
fn check_for_process(name: &str) -> bool {
    // Create a new System instance
    let sys = System::new_all();
    // Mutable boolean for the result
    let mut result = false;

    // Loop attempts to find if the process is running
    for process in sys.processes_by_exact_name(name.as_ref()) {
        if process.name() == name {
            result = true;
            break;
        }
    }

    result
}
