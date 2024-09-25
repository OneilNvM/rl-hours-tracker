//! The Rocket League Hours Tracker library contains modules which provide additional
//! functionality to the Rocket League Hours Tracker binary. This library is currently
//! implements the [`website_files`] module, which provides the functionality to generate
//! the Html, CSS, and JavaScript for the Rocket League Hours Tracker website.
//! 
//! The website functionality takes adavantage of the [`build_html`] library, which allows us
//! to generate the Html for the website, alongside the [`webbrowser`] library, which allows us
//! to open the website in a browser.
//! 
//! # Use Case
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
pub mod website_files;