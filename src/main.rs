use rl_hours_tracker::{create_directory, run, update_past_two};

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

    // Create the directories for the program
    let folders_result = create_directory();

    // Handles the successful result from the 'create_directory' function or panics if any errors occurred
    match folders_result {
        Ok(_) => {
            println!("All directories successfully created!");
        }
        Err(e) => panic!(
            "There was an error when creating the programs directories.\n Error Kind: {}\n{e}",
            e.kind()
        ),
    }

    // Updates the hours in the past two weeks if it returns true
    if update_past_two() {
        println!("Past Two Updated!\n");
    }

    run();
}
