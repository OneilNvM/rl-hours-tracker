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
//! ```
//!     println!("You got it Oneil :)");
//! ```
use build_html::HtmlTag;
use chrono::prelude::*;
use chrono::Duration as CDuration;
use std::io::ErrorKind;
use std::thread;
use std::u64;
use std::usize;
use std::{fs, io};
use std::{
    fs::File,
    io::{Read, Write},
    time::Duration,
};
use stopwatch::Stopwatch;
use sysinfo::System;
use build_html::{Html, HtmlPage, HtmlContainer, HtmlElement};
use webbrowser;

fn main() {
    // String reference of the Rocket League process name
    let process_name = "RocketLeague.exe";
    // Mutable boolean to determine when the program is waiting for the process to run
    let mut is_waiting = false;
    // Mutable string for user option
    let mut option = String::new();

    // Create the folder directory RLHoursFolder on the C: drive
    let folder = fs::create_dir("C:\\RLHoursFolder");

    // Match block to handle Ok and Err variants
    match folder {
        Ok(_) => println!("Rocket League Hours Folder created on C: Drive!"), // Successful folder creation
        Err(e) => {
            if e.kind() != ErrorKind::AlreadyExists {
                panic!(
                    "Folder was not created due to an error!\nError Kind: {}\n",
                    e.kind()
                );
            } // Failed folder creation
        }
    }

    // Updates the hours in the past two weeks if it returns true
    if update_past_two() {
        println!("Past Two Updated!\n");
    }

    // Run the main loop
    run_main_loop(process_name, &mut is_waiting, &mut option);
}

fn generate_website_html() {
    let index = File::create("C:\\RLHoursFolder\\index.html");
    let hours_file = File::open("C:\\RLHoursFolder\\hours.txt");
    let date_file = File::open("C:\\RLHoursFolder\\date.txt");

    let mut page = HtmlPage::new().with_title("Rocket League Hours Tracker");

    match index {
        Ok(mut idx_file) => {
            let mut hrs_content = String::new();
            let mut date_content = String::new();

            if let Ok(mut hrs_file) = hours_file {
                if let Ok(_) = hrs_file.read_to_string(&mut hrs_content) {
                    println!("Hours Content: {}", hrs_content);
                }
            }

            if let Ok(mut dt_file) = date_file {
                if let Ok(_) = dt_file.read_to_string(&mut date_content) {
                    println!("Date Content: {}", date_content);
                }
            }

            let mut hrs_lines: Vec<&str> = hrs_content.split("\n").collect();
            let mut date_lines: Vec<&str> = date_content.split("\n").collect();

            hrs_lines.pop();
            date_lines.pop();

            let main_heading = HtmlElement::new(HtmlTag::Heading1).with_child(hrs_lines.remove(0).into());
            let hours_heading = HtmlElement::new(HtmlTag::Heading2).with_child("Hours File".into());
            let date_heading = HtmlElement::new(HtmlTag::Heading2).with_child("Date File".into());

            let mut hours_div_elem = HtmlElement::new(HtmlTag::Div);
            let mut date_div_elem = HtmlElement::new(HtmlTag::Div);

            hours_div_elem.add_child(hours_heading.into());
            date_div_elem.add_child(date_heading.into());

            for line in hrs_lines {
                hours_div_elem.add_paragraph(line);
            }

            for line in date_lines {
                date_div_elem.add_paragraph(line);
            }

            page.add_raw(main_heading.to_html_string());

            page.add_raw(hours_div_elem.to_html_string());

            page.add_raw(date_div_elem.to_html_string());

            let contents = page.to_html_string();

            if let Ok(_) = idx_file.write_all(&contents.as_bytes()) {
                println!("Contents: {}", contents);

                let mut option = String::new();

                println!("Open hours website in browser (y/n)?");
                io::stdin().read_line(&mut option).unwrap();

                if option.trim() == "y" || option.trim() == "Y" {
                    if webbrowser::open("C:\\RLHoursFolder\\index.html").is_ok() {
                        println!("200 OK");
                    };
                }
            }
        }
        Err(e) => {
            panic!("There was an issue with html: {:?}", e);
        }
    }
}

/// This function runs the main loop of the program. This checks if the `RocketLeague.exe` process is running and
/// runs the `record_hours` function if it is running, otherwise it will continue to wait for the process to start.
fn run_main_loop(process_name: &str, is_waiting: &mut bool, option: &mut String) {
    // Main loop for the program
    'main_loop: loop {
        // Checks if the process is running
        if check_for_process(process_name) {
            // Begins the loop which records the seconds past after the process began
            record_hours(process_name);

            generate_website_html();

            // Change is_waiting value back to false
            *is_waiting = false;

            // Allow user to choose whether to continue the program or end it
            println!("End program (y/n)?");
            io::stdin().read_line(option).unwrap();

            // Check the option the user gave and respond accordingly
            if option.trim() == "y" || option.trim() == "Y" {
                break 'main_loop;
            } else if option.trim() == "n" || option.trim() == "N" {
                *option = String::new();
                continue;
            } else {
                println!("Unexpected input! Ending program.");
                break 'main_loop;
            }
        } else {
            // Print 'Waiting for Rocket League to start...' only once by changing the value of is_waiting to true
            if !*is_waiting {
                println!("Waiting for Rocket League to start...");
                *is_waiting = true;
            }
            // Sleep for 1000ms after every loop to save on CPU usage
            thread::sleep(Duration::from_millis(1000));
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

    println!("Rocket League is running");

    // Loop checks for when the process has ended
    loop {
        if !check_for_process(process_name) {
            // Stops the stopwatch
            sw.stop();

            // Stores the seconds elapsed as u64
            let seconds: u64 = sw.elapsed_ms() as u64 / 1000;
            // Stores the hours as f32
            let hours: f32 = (sw.elapsed_ms() as f32 / 1000_f32) / 3600_f32;

            // Opens both the hours file and date file in read mode if they exist
            let hours_result = File::open("C:\\RLHoursFolder\\hours.txt");
            let date_result = File::open("C:\\RLHoursFolder\\date.txt");

            write_to_date(date_result, &seconds);

            // Stores the hours past two by calling the calculate_past_two function and calculating the hours as f32
            let hours_buffer = calculate_past_two().unwrap_or(u64::MAX);

            if hours_buffer != u64::MAX {
                let hours_past_two = hours_buffer as f32 / 3600_f32;

                write_to_hours(hours_result, &seconds, &hours, &hours_past_two, &sw);
            } else {
                println!("The hours in the past two weeks was not calculated.");
            }

            break;
        }
        // Sleep for 1000ms at the end of every loop
        thread::sleep(Duration::from_millis(1000));
    }
}

/// This function updates the hours in the past two weeks in the `hours.txt` file.
/// The hours past two is calculated through the `calculate_past_two` function (Go to function for more details)
/// The function returns `true` if the new string has been written to `hours.txt`.
/// `false` is returned if `hours.txt` does not exist.
fn update_past_two() -> bool {
    // Open the 'hours.txt' file in read mode
    let hours_file_result = File::open("C:\\RLHoursFolder\\hours.txt");
    // Stores the calculated 'hours past two' as f32
    let hours_buffer = calculate_past_two().unwrap_or(u64::MAX);

    let hours_past_two ;

    if hours_buffer != u64::MAX {
        hours_past_two = hours_buffer as f32 / 3600_f32;
    } else {
        println!("Past two was not calculated.");
        return false
    }

    // Checks if the 'hours.txt' file exists, then stores the File in the mutable 'file' variable
    if let Ok(mut file) = hours_file_result {
        // Create a new empty String
        let mut hours_file_str = String::new();

        // Match statement returns the true if operation was successful or panics if there was an error
        let val: bool = match file.read_to_string(&mut hours_file_str) {
            Ok(_) => {
                // Deconstruct the seconds and hours tuple returned by the retrieve_time function
                let (seconds, hours) = retrieve_time(&hours_file_str);

                // Creates or truncates the 'hours.txt' file and opens it in write mode
                let write_hours_result = File::create("C:\\RLHoursFolder\\hours.txt");

                // Checks if the 'hours.txt' file was created, then stores the File in the mutable 'w_file' variable
                match write_hours_result {
                    Ok(mut w_file) => {
                        // Stores the new contents of the file as String
                        let rl_hours_str = format!("Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}\nHours Past Two Weeks: {:.1}hrs\n", seconds, hours, hours_past_two);

                        // Checks if writing to the file was successful
                        match w_file.write_all(&rl_hours_str.as_bytes()) {
                            // Return true to val
                            Ok(_) => {
                                generate_website_html();
                                true
                            },
                            // Panic if there was an error when writing to the file
                            Err(e) => panic!("Error occurred in 'update_past_two' function: There was an issue writing to 'hours.txt'.\nError Kind: {}", e.kind())
                        }
                    }
                    // Panic if there was an error creating the file
                    Err(e) => panic!(
                        "Error occurred in 'update_past_two' function: There was an issue with file creation.\nError kind: {}",
                        e.kind()
                    ),
                }
            }
            // Panic if there was an error reading the file
            Err(e) => {
                println!("Past Two not updated: {}", e.kind());
                false
            }
        };

        val
    } else {
        false
    }
}

/// This function takes the `contents: &String` parameter which contains the contents from the `hours.txt` file
/// and returns a tuple of `(u64, f32)` which contains the seconds and hours from the file.
fn retrieve_time(contents: &String) -> (u64, f32) {
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
        if num.is_numeric() {
            hrs_vec.push(num);
        } else if num == '.' {
            hrs_vec.push(num);
        }
    }

    // Collect the Vector characters as a String
    let seconds_str: String = sec_vec.iter().collect();
    let hours_str: String = hrs_vec.iter().collect();

    // Parse the seconds string as u64 and hours string as f32
    let old_seconds: u64 = seconds_str.parse().unwrap();
    let old_hours: f32 = hours_str.parse().unwrap();

    // Return a tuple of the old seconds and old hours
    (old_seconds, old_hours)
}

/// This function takes a reference of a [`Vec<&str>`] Vector and returns a [`usize`] as an index of the closest
/// after the date two weeks ago.
fn closest_date(split_newline: &Vec<&str>) -> usize {
    // Store the local date today
    let today = Local::now().date_naive();
    // Store the date two weeks ago
    let mut current_date = today - CDuration::days(14);

    // While loop attempts to find what date is closest to the date two weeks ago, within the Vector
    while current_date <= today {
        // date_binary_search takes a reference of split_newline Vector and the current iteration of the date
        let idx = date_binary_search(split_newline, &current_date.to_string());

        // Returns the index of the closest date if value is not usize::MAX
        if idx != usize::MAX {
            return idx;
        }

        // Increments the date
        current_date += CDuration::days(1);
    }

    // Return usize::MAX if there are any issues
    usize::MAX
}

/// This function is used to perform a binary search on a [`Vec<&str>`] Vector and compares the dates in the Vector with
/// the `c_date` [`String`]. The function then returns a [`usize`] for the index of the date, or a [`usize::MAX`] if the
/// date is not present.
fn date_binary_search(split_newline: &Vec<&str>, c_date: &String) -> usize {
    // Initialize mutable variable 'high' with last index of Vector
    let mut high = split_newline.len() - 1;
    // Initialize mutable variable 'low' to 0
    let mut low = 0;
    // Initialize mutable variable 'result' to 0
    let mut result = 0;
    // Initialize mutable variable 'is_zero' to true
    let mut is_zero = true;

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
            is_zero = false;
            break;
        } else if s_mid[0] < c_date {
            low = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        }
    }

    // While loop checks for any duplicates of the date two weeks ago in order to include them in the new Vector
    // This loop only runs if the date is found in the Vector
    while !is_zero {
        // Date Vector for current iteration
        let date_vec: Vec<&str> = split_newline[result].split_whitespace().collect();
        // Date string reference for the current iteration
        let date_str = date_vec[0];

        // Check if result is equal to zero and if the date is equal to the date two weeks ago
        // Return index if true
        if result == 0 && date_str == c_date {
            return result;
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
            return result;
        } else {
            result = ptr;
        }
    }

    // Return usize::MAX if the date is not found
    usize::MAX
}

/// This function calculates the hours recorded in the past two weeks and returns the total number of seconds as [`u64`]
/// The contents from `date.txt` are read and split by `\n` character and stored in a [`Vec<&str>`] Vector.
/// It is then ordered and looped through in order to compare the date to the current iteration of the date two weeks ago.
/// The seconds are retrieved from the dates that match the current date in the iteration of the while loop and the seconds
/// are added to `seconds_past_two` which is returned at the end of the function.
fn calculate_past_two() -> Option<u64> {
    // Open the 'date.txt' file in read mode
    let date_file_result = File::open("C:\\RLHoursFolder\\date.txt");
    // Initialize a mutable variable as u64 for the seconds past two
    let mut seconds_past_two: u64 = 0;

    // Checks if the 'date.txt' file was opened, then stores File in the mutable 'date_file' variable
    match date_file_result {
        Ok(mut date_file) => {
            // Creates and empty String
            let mut date_file_str = String::new();

            // Checks if the file was read successfully
            match date_file.read_to_string(&mut date_file_str) {
                Ok(_) => {
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

                    // Assign value from date_binary_search to variable
                    let date_idx = date_binary_search(&split_newline, &cur_date.to_string());

                    // Checks if the index returned from date_binary_search is usize::MAX
                    // Either sets the split_line_copy variable to a string reference slice of the Vector, with the first element
                    // as the first occurrence of the date two weeks ago
                    // Or, sets it to the closest date after the date two weeks ago
                    if date_idx != usize::MAX {
                        split_line_copy = &split_newline[date_idx..];
                    } else {
                        let closest = closest_date(&split_newline);

                        if closest != usize::MAX {
                            split_line_copy = &split_newline[closest..];
                        } else {
                            println!("The closest date could not be found");
                            return None
                        }
                    }

                    // While loop checks if the date is in the contents string and adds the seconds accompanied with it, to the seconds_past_two variable
                    while !is_after_today {
                        // Checks if the current iteration of the date two weeks ago is greater than today
                        if cur_date > today {
                            is_after_today = true;
                            continue;
                        }

                        // Loop through split_line_copy vector and compare the date to the cur_date
                        for date in split_line_copy.to_vec() {
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
                // Panics if there was an error reading the file
                Err(e) => panic!(
                    "Error occurred in 'calculate_past_two' function: There was an issue reading 'date.txt'.\nError Kind: {}",
                    e.kind()
                ),
            }
        }
        // Panics if there was an error opening the file
        Err(e) => println!("Past Two not calculated. Error Kind: {}", e.kind()),
    }
    Some(seconds_past_two)
}

/// This function constructs a new [`String`] which will have the contents to write to `hours.txt` with new hours and seconds
/// and returns it.
fn return_new_hours(contents: &String, seconds: &u64, hours: &f32, past_two: &f32) -> String {
    // Retrieves the old time and seconds from the contents String
    let time = retrieve_time(contents);

    // Deconstruct the seconds and hours from the tuple
    let (old_seconds, old_hours) = time;

    // Add the new seconds and the new hours to the old
    let added_seconds = old_seconds + *seconds;
    let added_hours = old_hours + *hours;

    // Return the new string of the file contents to be written to the file
    format!(
        "Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}\nHours Past Two Weeks: {:.1}hrs\n",
        added_seconds, added_hours, past_two
    )
}

/// This function writes the new contents to the `hours.txt` file. This includes the total `seconds`, `hours`, and `hours_past_two`.
fn write_to_hours(
    hours_result: Result<File, io::Error>,
    seconds: &u64,
    hours: &f32,
    hours_past_two: &f32,
    sw: &Stopwatch,
) {
    // Checks if the file exists, then stores the File into the mutable 'file' variable
    if let Ok(mut file) = hours_result {
        // Mutable variable to store file contents as a String
        let mut contents = String::new();

        // Checks if the file reads successfully, write new data to the file
        if let Ok(_) = file.read_to_string(&mut contents) {
            // Stores the new contents for the file as a String
            let rl_hours_str = return_new_hours(&contents, seconds, hours, hours_past_two);

            // Opens the file in write mode
            let truncated_file = File::create("C:\\RLHoursFolder\\hours.txt");

            // Checks if the file was exists, then stores the truncated File into the mutable 't_file' variable
            if let Ok(mut t_file) = truncated_file {
                // Checks if writing to the file was successful
                match t_file.write_all(&rl_hours_str.as_bytes()) {
                    Ok(_) => {
                        // Prints the String contents and the elapsed time
                        println!("{}", rl_hours_str);
                        println!("Elapsed Time: {}s", seconds);
                    }
                    // Panics if there was an error writing to the file
                    Err(e) => panic!(
                        "Error occurred in 'write_to_hours' function: There was an issue when truncating 'hours.txt'.\nError Kind: {}",
                        e.kind()
                    ),
                }
            }
        }
    } else {
        // Checks if the file was created successfully, then stores the File in the mutable 'file' variable
        match File::create("C:\\RLHoursFolder\\hours.txt") {
            Ok(mut file) => {
                // Store the total seconds, hours and the new String for the file in variables
                let total_seconds = sw.elapsed_ms() / 1000;
                let total_hours: f32 = (sw.elapsed_ms() as f32 / 1000_f32) / 3600_f32;
                let rl_hours_str = format!(
                                "Rocket League Hours\nTotal Seconds: {}s\nTotal Hours: {:.1}\nHours Past Two Weeks: {:.1}\n", total_seconds, total_hours, hours_past_two
                            );

                // Checks if writing to the file was successful
                match file.write_all(&rl_hours_str.as_bytes()) {
                    Ok(_) => println!("The hours file was successfully created"),
                    // Panic if there was any kind of error during writing process
                    Err(e) => panic!(
                        "Error occurred in 'write_to_hours' function: There was an issue writing to 'hours.txt'.\nError Kind: {}",
                        e.kind()
                    ),
                }
            }
            // Panic if there was an error when attempting to create the file
            Err(e) => panic!(
                "Error occurred in 'write_to_hours' function: There was an issue creating 'hours.txt'.\nError Kind: {}",
                e.kind()
            ),
        }
    }
}

/// This function writes new contents to the `date.txt` file. This uses the [`Local`] struct which allows us to use the [`Local::now()`]
/// function to retrieve the local date and time as [`DateTime<Local>`]. The date is then turned into a [`NaiveDate`] by using [`DateTime<Local>::date_naive()`]
/// which returns us the date by itself.
fn write_to_date(date_result: Result<File, io::Error>, seconds: &u64) {
    // Checks if the date file exists, then handles file operations
    if let Ok(_) = date_result {
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

                // Checks if writing to the file was successful
                match date_file.write_all(&today_str.as_bytes()) {
                    Ok(_) => println!("{}", today_str),
                    // Panics if there was an issue writing to the file
                    Err(e) => panic!(
                        "Error occurred in 'write_to_date' function: There was an issue appending to 'date.txt'.\nError Kind: {}",
                        e.kind()
                    ),
                }
            }
            // Panics if there was an issue opening the file
            Err(e) => panic!(
                "Error occurred in 'write_to_date' function: There was an issue opening 'date.txt'.\nError Kind: {}",
                e.kind()
            ),
        }
    } else {
        // Checks if the file was created, then stores the File in the mutable 'file' variable
        match File::create("C:\\RLHoursFolder\\date.txt") {
            Ok(mut file) => {
                // Store the current local date
                let today = Local::now().date_naive();

                // This String stores the date today, and the seconds elapsed in session
                let today_str = format!("{} {}s\n", today, seconds);

                // Checks if writing to the file was successful
                match file.write_all(&today_str.as_bytes()) {
                    Ok(_) => println!("The date file was successfully created"),
                    // Panics if there was an error writing to the file
                    Err(e) => panic!(
                        "Error occurred in 'write_to_date' function: There was an issue writing to 'date.txt'.\nError Kind: {}",
                        e.kind()
                    ),
                }
            }
            // Panics if there was an error creating the file
            Err(e) => panic!(
                "Error occurred in 'write_to_date' function: There was an issue creating 'date.txt'.\nErorr Kind: {}",
                e.kind()
            ),
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
