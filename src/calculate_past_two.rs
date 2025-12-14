//! Module contains functions for caclulating the hours in the past two weeks.
use std::{error::Error, fs::File, io::{Read, Write}};

use chrono::{prelude::*, Duration as CDuration};
use colour::{dark_red_ln_bold, green_ln_bold, yellow_ln_bold};
use log::{info, warn};

use crate::{PastTwoError, retrieve_time, website_files};

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
/// date is not present. The worst case for this search is O(L * log n).
pub fn date_binary_search(split_newline: &[&str], c_date: &String) -> Option<usize> {
    let mut high = split_newline.len() - 1;
    let mut date_found = false;
    let mut low = 0;
    let mut result = 0;

    while low <= high {
        let mid = low + (high - low) / 2;

        let s_mid: Vec<&str> = split_newline[mid].split_whitespace().collect();

        if s_mid[0] == c_date {
            if mid == 0 {
                break;
            }
            date_found = true;
            result = mid;
            high = mid - 1;
        } else if *s_mid[0] < **c_date {
            if date_found {
                break;
            }
            low = mid + 1;
        } else {
            if mid == 0 {
                return None;
            }
            high = mid - 1;
        }
    }

    if date_found {
        Some(result)
    } else {
        None
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
    // Check if the date.txt file exists
    let mut date_file_result = File::open("C:\\RLHoursFolder\\date.txt")?;
    let mut seconds_past_two: u64 = 0;

    info!("Calculate Past Two: START\n");
    let mut date_file_str = String::new();

    // Checks if the file was read successfully
    date_file_result.read_to_string(&mut date_file_str)?;

    yellow_ln_bold!("Dates retrieved...");
    let mut split_newline: Vec<&str> = date_file_str.split("\n").collect();
    split_newline.pop();
    split_newline.sort();

    let today = Local::now().date_naive();
    let two_weeks_ago = today - CDuration::days(14);
    let cur_date: NaiveDate = two_weeks_ago;

    // Declare variable for string reference slice
    let split_line_copy: &[&str];

    yellow_ln_bold!("Finding date two weeks ago...");
    let date_idx = date_binary_search(&split_newline, &cur_date.to_string());

    // Checks the value returned from date_binary_search
    match date_idx {
        Some(index) => {
            yellow_ln_bold!("Date found...");
            split_line_copy = &split_newline[index..];
        }
        // Find closest date if the target date was not found
        None => {
            dark_red_ln_bold!("\nDate not found. Searching for closest date...\n");
            let closest = closest_date(&split_newline);

            match closest {
                Some(index) => {
                    yellow_ln_bold!("Date found...");
                    split_line_copy = &split_newline[index..];
                }
                None => return Err(PastTwoError.into()),
            }
        }
    }

    yellow_ln_bold!("Calculating past two...");
    sum_total_seconds(split_line_copy, cur_date, today, &mut seconds_past_two)?;

    green_ln_bold!("Past two calculated\n");
    info!("Calculate Past Two: FINISHED\n");
    Ok(seconds_past_two)
}

fn sum_total_seconds(
    split_line_copy: &[&str],
    mut cur_date: NaiveDate,
    today: NaiveDate,
    seconds_past_two: &mut u64,
) -> Result<(), Box<dyn Error>> {
    let mut is_after_today = false;
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
                let total_seconds: u64 = seconds_str.parse()?;

                // Add the total seconds to the seconds_past_two variable
                *seconds_past_two += total_seconds;
            }
        }

        // Increase the current date to the next day
        cur_date += CDuration::days(1);
    }

    Ok(())
}

/// This function updates the hours in the past two weeks in the `hours.txt` file.
/// The hours past two is calculated through the [`calculate_past_two`] function
/// The function returns a [`Result<bool>`] if the function was able to successfully
/// update the hours past two, and write it to the `hours.txt` file.
///
/// # Errors
/// Returns an [`io::Error`] if there were any issues with file operations.
pub fn update_past_two() -> Result<bool, Box<dyn Error>> {
    // Checks if the 'hours.txt' file exists
    let mut hours_file_result = File::open("C:\\RLHoursFolder\\hours.txt")?;

    // Buffer which stores the hours in the past two weeks
    let hours_buffer = calculate_past_two().unwrap_or_else(|e| {
        warn!("failed to calculate past two: {e}\n");
        0
    });

    // Check the value of the buffer
    let hours_past_two: f32 = if hours_buffer != 0 {
        hours_buffer as f32 / 3600_f32
    } else {
        warn!("past two returned zero seconds");
        return Ok(false);
    };

    let mut hours_file_str = String::new();

    // Attempt to read from old hours.txt file
    hours_file_result.read_to_string(&mut hours_file_str)?;

    let (seconds, hours) = retrieve_time(&hours_file_str)?;

    // Attempt to write to the file
    let mut write_hours_result = File::create("C:\\RLHoursFolder\\hours.txt")?;

    let rl_hours_str = format!("Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}hrs\nHours Past Two Weeks: {:.1}hrs\n", seconds, hours, hours_past_two);

    // Check if the write succeeds
    write_hours_result.write_all(rl_hours_str.as_bytes())?;

    // Update website files
    website_files::generate_website_files(false)
        .unwrap_or_else(|e| warn!("failed to generate website files: {e}"));
    Ok(true)
}
