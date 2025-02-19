use std::process;
use std::{env, io::ErrorKind};

use rl_hours_tracker::{create_directory, run, run_self_update, update_past_two};

fn main() {
    println!(
        "
   ___           __       __    __                         
  / _ \\___  ____/ /_____ / /_  / /  ___ ___ ____ ___ _____ 
 / , _/ _ \\/ __/  '_/ -_) __/ / /__/ -_) _ `/ _ `/ // / -_)
/_/|_|\\___/\\__/_/\\_\\\\__/\\__/ /____/\\__/\\_,_/\\_, /\\_,_/\\__/ 
   __ __                    ______         /___/_          
  / // /__  __ _________   /_  __/______ _____/ /_____ ____
 / _  / _ \\/ // / __(_-<    / / / __/ _ `/ __/  '_/ -_) __/
/_//_/\\___/\\_,_/_/ /___/   /_/ /_/  \\_,_/\\__/_/\\_\\\\__/_/   
                                                           
"
    );

    // Checks if the program is being run from the AppData directory.
    // This is done to make sure that anyone installing the binary through
    // cargo does not run the self update functionality, as they can update
    // the binary through cargo.
    if let Ok(path) = env::current_dir() {
        let dir = path.to_str().unwrap();

        if dir.contains("AppData") {
            run_self_update().unwrap_or_else(|e| eprintln!("error running self update: {e}"));
        }
    }

    // Create the directories for the program
    let folders_result = create_directory();

    // Handles the successful result from the 'create_directory' function or panics if any errors occurred
    if !folders_result.is_empty() {
        for folder in folders_result {
            folder.unwrap_or_else(|e| {
                if e.kind() != ErrorKind::AlreadyExists {
                    eprintln!("There was an issue when creating folders: {e}");
                    process::exit(1);
                }
            })
        }
    } else {
        println!("All directories created successfully!");
    }

    // Updates the hours in the past two weeks if it returns true
    if update_past_two().unwrap_or_else(|e| {
        eprintln!("past two could not be updated: {e}");
        false
    }) {
        println!("Past Two Updated!\n");
    }

    run();
}
