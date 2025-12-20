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
use chrono::Local;
use colour::{black_bold, blue_ln_bold, cyan, green, green_ln_bold, red, white, yellow_ln_bold};
use log::{error, info, warn, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config, Handle,
};
use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::{self, Read, Write},
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime},
};
use stopwatch::Stopwatch;
use sysinfo::System;
use tokio::runtime::Runtime;

use crate::calculate_past_two::calculate_past_two;

pub mod calculate_past_two;
#[cfg(test)]
mod tests;
pub mod update;
pub mod website_files;
pub mod winit_tray_icon;

/// Type alias for Results which only return [`std::io::Error`] as its error variant.
pub type IoResult<T> = Result<T, io::Error>;

/// Contains the relevant data for running the program
struct ProgramRunVars {
    process_name: String,
    is_waiting: bool,
    option: String,
    currently_tracking: Arc<Mutex<AtomicBool>>,
    stop_tracker: Arc<Mutex<AtomicBool>>,
}

impl ProgramRunVars {
    fn new(
        stop_tracker: Arc<Mutex<AtomicBool>>,
        currently_tracking: Arc<Mutex<AtomicBool>>,
    ) -> Self {
        Self {
            process_name: String::from("RocketLeague.exe"),
            is_waiting: false,
            option: String::with_capacity(3),
            stop_tracker,
            currently_tracking,
        }
    }
}

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

/// Initializes logging configuration for the program
///
/// Logs are stored in `C:/RLHoursFolder/logs`
pub fn initialize_logging() -> Result<Handle, Box<dyn Error>> {
    // Create appenders
    let stdout = ConsoleAppender::builder().build();
    let general_logs = FileAppender::builder()
        .build("C:/RLHoursFolder/logs/general_$TIME{%Y-%m-%d_%H-%M-%S}.log")?;
    let wti_logs = FileAppender::builder()
        .build("C:/RLHoursFolder/logs/tray-icon_$TIME{%Y-%m-%d_%H-%M-%S}.log")?;
    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("C:/RLHoursFolder/logs/requests.log")?;

    // Create loggers
    let rl_hours_tracker_logger = Logger::builder()
        .additive(false)
        .appenders(vec!["general_logs"])
        .build("rl_hours_tracker", LevelFilter::Info);
    let rl_hours_tracker_update_logger = Logger::builder()
        .additive(false)
        .appenders(vec!["requests", "general_logs"])
        .build("rl_hours_tracker::update", LevelFilter::Trace);
    let rl_hours_tracker_cpt_logger = Logger::builder()
        .additive(false)
        .appenders(vec!["general_logs"])
        .build("rl_hours_tracker::calculate_past_two", LevelFilter::Info);
    let rl_hours_tracker_wti_logger = Logger::builder()
        .additive(false)
        .appenders(vec!["wti_logs"])
        .build("rl_hours_tracker::winit_tray_icon", LevelFilter::Info);

    // Move loggers and appenders into vectors
    let loggers = vec![
        rl_hours_tracker_logger,
        rl_hours_tracker_update_logger,
        rl_hours_tracker_cpt_logger,
        rl_hours_tracker_wti_logger,
    ];
    let appenders = vec![
        Appender::builder().build("stdout", Box::new(stdout)),
        Appender::builder().build("general_logs", Box::new(general_logs)),
        Appender::builder().build("requests", Box::new(requests)),
        Appender::builder().build("wti_logs", Box::new(wti_logs)),
    ];

    let config = Config::builder()
        .appenders(appenders)
        .loggers(loggers)
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))?;

    // Initialize logging configuration
    let handle = log4rs::init_config(config)?;

    Ok(handle)
}

/// This runs the [`update::check_for_update`] function
pub fn run_self_update() -> Result<(), Box<dyn Error>> {
    let rt = Runtime::new()?;

    rt.block_on(update::check_for_update())?;

    Ok(())
}

/// This function runs the program
pub fn run(stop_tracker: Arc<Mutex<AtomicBool>>, currently_tracking: Arc<Mutex<AtomicBool>>) {
    let mut program = ProgramRunVars::new(stop_tracker, currently_tracking);

    // Run the main loop
    run_main_loop(&mut program);
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
fn run_main_loop(program: &mut ProgramRunVars) {
    loop {
        // Check if the process is running
        if check_for_process(&program.process_name) {
            record_hours(
                &program.process_name,
                program.stop_tracker.clone(),
                program.currently_tracking.clone(),
            );

            // Generate the website files
            website_files::generate_website_files(true)
                .unwrap_or_else(|e| warn!("failed to generate website files: {e}"));

            program.is_waiting = false;

            print!("End program (");
            green!("y");
            print!(" / ");
            red!("n");
            print!("): ");
            std::io::stdout()
                .flush()
                .unwrap_or_else(|_| println!("End program (y/n)?\n"));
            io::stdin()
                .read_line(&mut program.option)
                .unwrap_or_default();

            if program.option.trim() == "y" || program.option.trim() == "Y" {
                print!("{}[2K\r", 27 as char);
                std::io::stdout()
                    .flush()
                    .expect("could not flush the output stream");
                yellow_ln_bold!("Goodbye!");
                process::exit(0);
            } else if program.option.trim() == "n" || program.option.trim() == "N" {
                program.option = String::with_capacity(3);
                continue;
            } else {
                error!("Unexpected input! Ending program.");
                process::exit(0)
            }
        } else {
            // Print 'Waiting for Rocket League to start...' only once by changing the value of is_waiting to true
            if !program.is_waiting {
                green!("Waiting for Rocket League to start.\r");
                io::stdout()
                    .flush()
                    .expect("could not flush the output stream");
                thread::sleep(Duration::from_millis(500));
                white!("Waiting for Rocket League to start..\r");
                io::stdout()
                    .flush()
                    .expect("could not flush the output stream");
                thread::sleep(Duration::from_millis(500));
                black_bold!("Waiting for Rocket League to start...\r");
                io::stdout()
                    .flush()
                    .expect("could not flush the output stream");
                thread::sleep(Duration::from_millis(500));
                print!("{}[2K\r", 27 as char);
                red!("Waiting for Rocket League to start\r");
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
fn record_hours(
    process_name: &str,
    stop_tracker: Arc<Mutex<AtomicBool>>,
    currently_tracking: Arc<Mutex<AtomicBool>>,
) {
    let mut sw = Stopwatch::start_new();

    blue_ln_bold!("\nRocket League is running\n");

    *currently_tracking.try_lock().unwrap_or_else(|e| {
        error!("error when attempting to access lock for currently_tracking: {e}");
        panic!("could not access lock for currently_tracking");
    }) = true.into();

    // Start live stopwatch
    live_stopwatch(process_name, stop_tracker.clone());

    *currently_tracking.try_lock().unwrap_or_else(|e| {
        error!("error when attempting to access lock for currently_tracking: {e}");
        panic!("could not access lock for currently_tracking");
    }) = false.into();

    *stop_tracker.try_lock().unwrap_or_else(|e| {
        error!("error when attempting to access lock for stop_tracking: {e}");
        panic!("could not access lock for stop_tracking");
    }) = false.into();

    // Stop the stopwatch
    sw.stop();

    info!("Record Hours: START\n");

    let seconds: u64 = sw.elapsed_ms() as u64 / 1000;
    let hours: f32 = (sw.elapsed_ms() as f32 / 1000_f32) / 3600_f32;

    let hours_result = File::open("C:\\RLHoursFolder\\hours.txt");
    let date_result = File::open("C:\\RLHoursFolder\\date.txt");

    // Write date and seconds to date.txt
    write_to_date(date_result, &seconds).unwrap_or_else(|e| {
        error!("error writing to date.txt: {e}");
        process::exit(1);
    });

    // Buffer which stores the hours in the past two weeks
    let hours_buffer = calculate_past_two().unwrap_or_else(|e| {
        warn!("failed to calculate past two: {e}");
        0
    });

    if hours_buffer != 0 {
        let hours_past_two = hours_buffer as f32 / 3600_f32;

        write_to_hours(hours_result, &seconds, &hours, &hours_past_two, &sw).unwrap_or_else(|e| {
            error!("error writing to hours.txt: {e}");
            process::exit(1);
        });
        info!("Record Hours: FINISHED\n")
    } else {
        warn!("past two returned zero seconds")
    }
}

fn live_stopwatch(process_name: &str, stop_tracker: Arc<Mutex<AtomicBool>>) {
    let mut timer_early = SystemTime::now();

    let mut seconds: u8 = 0;
    let mut minutes: u8 = 0;
    let mut hours: u16 = 0;

    while check_for_process(process_name)
        && stop_tracker
            .try_lock()
            .unwrap_or_else(|e| {
                error!("error when attempting to access lock for stop_tracking: {e}");
                panic!("could not access lock for stop_tracking");
            })
            .fetch_not(Ordering::SeqCst)
    {
        let timer_now = timer_early
            .checked_add(Duration::from_millis(999))
            .unwrap_or_else(|| {
                error!("could not return system time");
                SystemTime::now()
            });

        let delay = timer_now.duration_since(timer_early).unwrap_or_else(|e| {
            warn!(
                "system time is ahead of the timer. SystemTime difference: {:?}",
                e.duration()
            );
            Duration::from_millis(1000)
        });

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
            cyan!("Time Elapsed: 0{}:0{}:0{}\r", hours, minutes, seconds);
        } else if hours >= 10 {
            if minutes < 10 && seconds < 10 {
                cyan!("Time Elapsed: {}:0{}:0{}\r", hours, minutes, seconds);
            } else if minutes < 10 && seconds >= 10 {
                cyan!("Time Elapsed: {}:0{}:{}\r", hours, minutes, seconds);
            } else if minutes >= 10 && seconds < 10 {
                cyan!("Time Elapsed: {}:{}:0{}\r", hours, minutes, seconds);
            } else {
                cyan!("Time Elapsed: {}:{}:{}\r", hours, minutes, seconds);
            }
        } else if hours < 10 && minutes >= 10 && seconds < 10 {
            cyan!("Time Elapsed: 0{}:{}:0{}\r", hours, minutes, seconds);
        } else if hours < 10 && minutes < 10 && seconds >= 10 {
            cyan!("Time Elapsed: 0{}:0{}:{}\r", hours, minutes, seconds);
        } else {
            cyan!("Time Elapsed: 0{}:{}:{}\r", hours, minutes, seconds);
        }

        // Flush the output
        io::stdout()
            .flush()
            .unwrap_or_else(|_| warn!("could not flush output stream"));

        thread::sleep(delay);

        timer_early += Duration::from_millis(999)
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

/// This function constructs a new [`String`] which will have the contents to write to `hours.txt` with new hours and seconds
/// and returns it.
fn return_new_hours(
    contents: &str,
    seconds: &u64,
    hours: &f32,
    past_two: &f32,
) -> Result<String, Box<dyn Error>> {
    yellow_ln_bold!("Getting old hours...");
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
        file.read_to_string(&mut contents)?;

        // Stores the new contents for the file as a String
        let rl_hours_str = return_new_hours(&contents, seconds, hours, hours_past_two)?;

        // Attempt to write to hours.txt
        let mut truncated_file = File::create("C:\\RLHoursFolder\\hours.txt")?;

        yellow_ln_bold!("Writing to hours.txt...");

        // Check if the write was successful
        truncated_file.write_all(rl_hours_str.as_bytes())?;

        green_ln_bold!("Successful!\n");
        Ok(())
    } else {
        // Check if the file was created successfully
        let mut file = File::create("C:\\RLHoursFolder\\hours.txt")?;
        let total_seconds = sw.elapsed_ms() / 1000;
        let total_hours: f32 = (sw.elapsed_ms() as f32 / 1000_f32) / 3600_f32;
        let rl_hours_str = format!(
                                "Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}hrs\nHours Past Two Weeks: {:.1}hrs\n", total_seconds, total_hours, hours_past_two
                            );

        yellow_ln_bold!("Writing to hours.txt...");

        // Checks if the write was successful
        file.write_all(rl_hours_str.as_bytes())?;

        green_ln_bold!("The hours file was successfully created");
        Ok(())
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
        let mut append_date_result = File::options()
            .append(true)
            .open("C:\\RLHoursFolder\\date.txt")?;

        // Attenot to open the date.txt file
        let today = Local::now().date_naive();

        let today_str = format!("{} {}s\n", today, seconds);

        yellow_ln_bold!("Appending to date.txt...");

        // Checks if the write was successful
        append_date_result.write_all(today_str.as_bytes())?;

        green_ln_bold!("Successful!\n");
        Ok(())
    } else {
        // Check if the file was created
        let mut file = File::create("C:\\RLHoursFolder\\date.txt")?;
        let today = Local::now().date_naive();

        let today_str = format!("{} {}s\n", today, seconds);

        yellow_ln_bold!("Appending to date.txt...");

        // Checks if the write was successful
        file.write_all(today_str.as_bytes())?;

        green_ln_bold!("The date file was successfully created");
        Ok(())
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
