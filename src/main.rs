use std::io::{ErrorKind, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, process, thread};

use colour::{blue, blue_ln, cyan, e_red_ln, green_ln, green_ln_bold};
use log::{error, warn};
use rl_hours_tracker::initialize_logging;
use rl_hours_tracker::winit_tray_icon::initialize_tray_icon;
use rl_hours_tracker::{
    calculate_past_two::update_past_two, create_directory, run, run_self_update,
};

fn main() {
    blue!(
        "

   ___           __       __    __                         
  / _ \\___  ____/ /_____ / /_  / /  ___ ___ ____ ___ _____ 
 / , _/ _ \\/ __/  '_/ -_) __/ / /__/ -_) _ `/ _ `/ // / -_)
/_/|_|\\___/\\__/_/\\_\\\\__/\\__/ /____/\\__/\\_,_/\\_, /\\_,_/\\__/ 
    "
    );
    cyan!(
        "
   __ __                    ______             __          
  / // /__  __ _________   /_  __/______ _____/ /_____ ____
 / _  / _ \\/ // / __(_-<    / / / __/ _ `/ __/  '_/ -_) __/
/_//_/\\___/\\_,_/_/ /___/   /_/ /_/  \\_,_/\\__/_/\\_\\\\__/_/   
                                                           
"
    );

    std::io::stdout().flush().unwrap_or_else(|_| {
        blue_ln!(
            "
        
   ___           __       __    __                         
  / _ \\___  ____/ /_____ / /_  / /  ___ ___ ____ ___ _____ 
 / , _/ _ \\/ __/  '_/ -_) __/ / /__/ -_) _ `/ _ `/ // / -_)
/_/|_|\\___/\\__/_/\\_\\\\__/\\__/ /____/\\__/\\_,_/\\_, /\\_,_/\\__/ 

   __ __                    ______             __          
  / // /__  __ _________   /_  __/______ _____/ /_____ ____
 / _  / _ \\/ // / __(_-<    / / / __/ _ `/ __/  '_/ -_) __/
/_//_/\\___/\\_,_/_/ /___/   /_/ /_/  \\_,_/\\__/_/\\_\\\\__/_/  
        "
        )
    });

    initialize_logging().unwrap_or_else(|e| {
        e_red_ln!("an error occurred when initializing logging: {e}");
        thread::sleep(Duration::from_secs(2));
        e_red_ln!("this program will end in 3 seconds");
        thread::sleep(Duration::from_secs(3));
        process::exit(1);
    });

    let stop_tracker: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    initialize_tray_icon(stop_tracker.clone());

    // Checks if the program is being run from the AppData directory.
    // This does not run the self update if using through rust binary.
    if let Ok(path) = env::current_dir() {
        let dir = path.to_str().unwrap_or_default();

        if dir.contains("AppData") {
            run_self_update().unwrap_or_else(|e| error!("error running self update: {e}"));
        }
    }

    // Create the directories for the program
    let folders_result = create_directory();

    // Handles the successful result from the 'create_directory' function or panics if any errors occurred
    if !folders_result.is_empty() {
        for folder in folders_result {
            folder.unwrap_or_else(|e| {
                if e.kind() != ErrorKind::AlreadyExists {
                    error!("There was an issue when creating folders: {e}");
                    process::exit(1);
                }
            })
        }
    } else {
        green_ln!("All directories created successfully!");
    }

    // Updates the hours in the past two weeks if it returns true
    if update_past_two().unwrap_or_else(|e| {
        warn!("past two could not be updated: {e}");
        false
    }) {
        green_ln_bold!("Past Two Updated!\n");
    }

    run(stop_tracker.clone());
}
