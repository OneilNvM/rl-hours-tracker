//! This module contains the functionality to generate the Html, CSS, and JavaScript for the
//! Rocket League Hours Tracker website.
use crate::IoResult;
use build_html::{Container, ContainerType, Html, HtmlContainer, HtmlElement, HtmlPage, HtmlTag};
use bytes::Bytes;
use colour::{green, green_ln_bold, red};
use log::{error, warn};
use reqwest::{Client, Response};
use std::{
    error::Error as ErrorTrait,
    fs::{write, File},
    io::{self, Error, ErrorKind, Read, Write},
    process,
    slice::Iter,
};
use tokio::runtime::Runtime;
use webbrowser;

/// The Github repository and the `Url` to the files in the repository.
#[derive(Debug, Clone)]
pub struct Github<'a> {
    owner: &'a str,
    repo: &'a str,
    branch: &'a str,
    path: &'a str,
    file: &'a str,
    url: String,
}

impl<'a> Github<'a> {
    /// Creates a new instance with empty strings.
    pub fn new(
        owner: &'a str,
        repo: &'a str,
        branch: &'a str,
        path: &'a str,
        file: &'a str,
    ) -> Github<'a> {
        Github {
            owner,
            repo,
            branch,
            path,
            file,
            url: String::new(),
        }
    }

    /// Gets the built url of the GitHub instance.
    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    /// Builds the `Url` for the raw contents of a file.
    ///
    /// This function should only be used for files on Github which can be opened in raw format.
    ///
    /// ## Usage
    ///
    /// ```
    ///let mut github_repo = Github::new("OneilNvM", "rl-hours-tracker", "master", "src", "main.rs");
    ///
    /// // Example Output: "https://raw.githubusercontent.com/OneilNvM/rl-hours-tracker/refs/heads/master/src/main.rs"
    /// github_repo.build_url();
    /// ```
    pub fn build_url(&mut self) {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/refs/heads/{}/{}/{}",
            self.owner, self.repo, self.branch, self.path, self.file
        );
        self.url = url;
    }

    /// Builds the `Url` for the blob of an image file.
    ///
    /// This function should only be used for image files in a Github repository.
    ///
    /// ## Usage
    ///
    /// ```
    ///let mut github_repo = Github::new("OneilNvM", "rl-hours-tracker", "master", "images", "img.png");
    ///
    /// // Example Output: "https://github.com/OneilNvM/rl-hours-tracker/blob/master/images/img.png"
    /// github_repo.build_image_url();
    /// ```
    pub fn build_image_url(&mut self) {
        let url = format!(
            "https://github.com/{}/{}/blob/{}/{}/{}",
            self.owner, self.repo, self.branch, self.path, self.file
        );
        self.url = url;
    }
}

/// This stores the file and image responses from the `GET` requests to GitHub
#[derive(Debug, Clone)]
pub struct GHResponse {
    raw_url: Vec<String>,
    image_url: Vec<Bytes>,
}

impl GHResponse {
    /// Creates a new instance
    pub fn new(raw_url: Vec<String>, image_url: Vec<Bytes>) -> GHResponse {
        GHResponse { raw_url, image_url }
    }
}

/// Sends a HTTP `GET` request and returns the response.
///
/// ## Usage
///
/// ```
/// let response = send_request(url).await;
///
/// let text = response.text().await;
/// ```
pub async fn send_request(url: &String) -> Response {
    // Construct a new client instance
    let client = Client::new();

    // Send the GET request
    let request = client.get(url).send().await;

    // Handle the request
    match request {
        Ok(response) => response,
        Err(e) => {
            error!("error sending get request for url: {url}\n{e}");
            process::exit(1);
        }
    }
}

/// Handles the response received from [`send_request`].
///
/// This function specifically handles the Urls from the [`Github`] instance, which was created
/// through [`Github::build_url`].
pub async fn handle_response(urls: Vec<String>) -> Vec<String> {
    let mut text_vec: Vec<String> = Vec::new();

    // Loop through the Urls
    for url in urls {
        let response = send_request(&url).await;

        let text = response.text().await;

        // Handle the response text
        match text {
            Ok(result) => text_vec.push(result),
            Err(e) => {
                error!("error retrieving full response text: {e}");
                process::exit(1);
            }
        }
    }

    text_vec
}

/// Handles the response received from [`send_request`].
///
/// This function specifically handles the urls from the [`Github`] instance, which was created
/// through [`Github::build_image_url`].
pub async fn handle_image_response(urls: Vec<String>) -> Vec<Bytes> {
    let mut blob_vec: Vec<Bytes> = Vec::new();

    // Loop through the Urls
    for url in urls {
        let response = send_request(&url).await;

        let blob = response.bytes().await;

        // Handle the response bytes
        match blob {
            Ok(result) => blob_vec.push(result),
            Err(e) => {
                error!("error retrieving response bytes: {e}");
                process::exit(1);
            }
        }
    }

    blob_vec
}

/// Runs the asynchronous functions to completion and returns a [`GHResponse`] instance.
///
/// This creates a new [`Runtime`] instance and runs the async functions to completion with [`Runtime::block_on`].
pub fn run_async_functions(
    urls1: Vec<String>,
    urls2: Vec<String>,
) -> Result<GHResponse, Box<dyn ErrorTrait>> {
    let rt = Runtime::new()?;

    // Run the async functions
    let result1 = rt.block_on(handle_response(urls1));
    let result2 = rt.block_on(handle_image_response(urls2));

    Ok(GHResponse::new(result1, result2))
}

/// This function is used to generate the necessary files for the Rocket League Hours Tracker website.
/// It accepts a bool [`bool`] as an argument which determines whether the option to open the website
/// in the browser should appear or not.
///
/// # Errors
/// Returns an [`io::Error`] if there were any file operations which failed
pub fn generate_website_files(boolean: bool) -> Result<(), Box<dyn ErrorTrait>> {
    // Create Github instances for the website files
    let mut github_main_css = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/css",
        "main.css",
    );
    let mut github_home_css = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/css",
        "home.css",
    );
    let mut github_animations_js = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/js",
        "animations.js",
    );
    let mut github_grey_icon = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/images",
        "rl-icon-grey.png",
    );
    let mut github_white_icon = Github::new(
        "OneilNvM",
        "rl-hours-tracker",
        "master",
        "website/images",
        "rl-icon-white.png",
    );

    // Build the Urls for the Github instances
    github_main_css.build_url();
    github_home_css.build_url();
    github_animations_js.build_url();
    github_grey_icon.build_url();
    github_white_icon.build_url();

    let github_text_vec = vec![
        github_main_css.url,
        github_home_css.url,
        github_animations_js.url,
    ];

    let github_blob_vec = vec![github_grey_icon.url, github_white_icon.url];

    // Run asynchronous functions and return a GHResponse instance
    let ghresponse = run_async_functions(github_text_vec, github_blob_vec)?;

    let mut bytes_iter = ghresponse.image_url.iter();
    let mut raw_iter = ghresponse.raw_url.iter();

    // Write the image bytes
    write(
        "C:\\RLHoursFolder\\website\\images\\rl-icon-grey.png",
        bytes_iter.next().unwrap(),
    )
    .unwrap_or_else(|e| warn!("failed to write rl-icon-grey.png: {e}"));
    write(
        "C:\\RLHoursFolder\\website\\images\\rl-icon-white.png",
        bytes_iter.next().unwrap(),
    )
    .unwrap_or_else(|e| warn!("failed to write rl-icon-white.png: {e}"));

    // Create the files for the website
    create_website_files(&mut raw_iter, boolean)
}

fn create_website_files(
    raw_iter: &mut Iter<'_, String>,
    boolean: bool,
) -> Result<(), Box<dyn ErrorTrait>> {
    // Create and open files
    let mut index = File::create("C:\\RLHoursFolder\\website\\pages\\index.html")?;
    let main_styles = File::create("C:\\RLHoursFolder\\website\\css\\main.css");
    let home_styles = File::create("C:\\RLHoursFolder\\website\\css\\home.css");
    let animations_js = File::create("C:\\RLHoursFolder\\website\\js\\animations.js");
    let mut hours_file = File::open("C:\\RLHoursFolder\\hours.txt");
    let mut date_file = File::open("C:\\RLHoursFolder\\date.txt");

    // Creates the main.css file
    match main_styles {
        Ok(mut ms_file) => {
            // Writes the CSS content to the file
            match ms_file.write_all(raw_iter.next().unwrap().as_bytes()) {
                Ok(_) => (),
                Err(e) => warn!("failed to write to main.css: {e}"),
            }
        }
        Err(e) => warn!("failed to create main.css: {e}"),
    }

    // Creates the home.css file
    match home_styles {
        Ok(mut hs_file) => {
            // Writes the CSS content to the file
            match hs_file.write_all(raw_iter.next().unwrap().as_bytes()) {
                Ok(_) => (),
                Err(e) => warn!("failed to write to home.css: {e}"),
            }
        }
        Err(e) => warn!("failed to create home.css: {e}"),
    }

    // Creates the animations.js file
    match animations_js {
        Ok(mut a_js_file) => {
            // Writes the JavaScript content to the file
            match a_js_file.write_all(raw_iter.next().unwrap().as_bytes()) {
                Ok(_) => (),
                Err(e) => warn!("failed to write to animations.js: {e}"),
            }
        }
        Err(e) => warn!("failed to create animations.js: {e}"),
    }

    // Generate the website
    let contents: String = generate_page(&mut hours_file, &mut date_file)?;

    // Initialize the 'contents' variable with the Html
    let page = contents.replace("<body>", "<body class=\"body adaptive\">");

    // Writes the index.html file
    index.write_all(page.as_bytes())?;

    // Prompt the user with the option to open the website
    if boolean {
        let mut option = String::new();

        print!("Open hours website in browser (");
        green!("y");
        print!(" / ");
        red!("n");
        print!("): ");
        io::stdout()
            .flush()
            .unwrap_or_else(|_| println!("Open hours website in browser (y/n)?"));
        io::stdin().read_line(&mut option).unwrap();

        if option.trim().to_lowercase() == "y"
            && webbrowser::open("C:\\RLHoursFolder\\website\\pages\\index.html").is_ok()
        {
            green_ln_bold!("OK\n");
        }
    }

    Ok(())
}

/// This function generates the necessary Html for the website via the [`build_html`] library. The `hours_file` and `date_file`
/// parameters are both mutable [`Result<File>`] references which provides us with a [`File`] if it is successful, or [`io::Error`] if
/// it fails. This function then returns a [`Result<String>`] of the Html.
///
/// # Errors
/// This function returns an [`io::Error`] if there were any errors during file operations.
fn generate_page(
    hours_file: &mut IoResult<File>,
    date_file: &mut IoResult<File>,
) -> IoResult<String> {
    let mut page = HtmlPage::new()
    .with_title("Rocket League Hours Tracker")
    .with_meta(vec![("charset", "UTF-8")])
    .with_meta(vec![("name", "viewport"), ("content", "width=device-width, initial-scale=1.0")])
    .with_head_link("../css/main.css", "stylesheet")
    .with_head_link("../css/home.css", "stylesheet")
    .with_head_link("https://fonts.googleapis.com", "preconnect")
    .with_head_link_attr("https://fonts.gstatic.com", "preconnect", [("crossorigin", "")])
    .with_head_link("https://fonts.googleapis.com/css2?family=Bebas+Neue&display=swap", "stylesheet")
    .with_head_link("https://fonts.googleapis.com/css2?family=Bebas+Neue&family=Oswald:wght@200..700&display=swap", "stylesheet")
    .with_script_link("../js/animations.js");

    page.add_container(
        Container::new(ContainerType::Div)
            .with_attributes(vec![("class", "animation-div adaptive")])
            .with_raw(""),
    );

    let mut hrs_content = String::new();
    let mut date_content = String::new();

    if let Ok(ref mut hrs_file) = hours_file {
        if hrs_file.read_to_string(&mut hrs_content).is_err() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "The files contents are not valid UTF-8.",
            ));
        }
    } else {
        return Err(Error::new(ErrorKind::NotFound, "The file 'hours.txt' could not be opened. Either it does not exist or it is not in the 'RLHoursFolder' directory."));
    }

    if let Ok(ref mut dt_file) = date_file {
        if dt_file.read_to_string(&mut date_content).is_err() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "The files contents are not valid UTF-8.",
            ));
        }
    } else {
        return Err(Error::new(ErrorKind::NotFound, "The file 'hours.txt' could not be opened. Either it does not exist or it is not in the 'RLHoursFolder' directory."));
    }

    let mut hrs_lines: Vec<&str> = hrs_content.split("\n").collect();
    let mut date_lines: Vec<&str> = date_content.split("\n").collect();

    hrs_lines.pop();
    date_lines.pop();

    let main_heading_vec: Vec<&str> = hrs_lines.remove(0).split_whitespace().collect();

    let main_heading = format!(
        "{} {}<br>{} Tracker",
        main_heading_vec[0], main_heading_vec[1], main_heading_vec[2]
    );

    page.add_container(
        Container::new(ContainerType::Header)
            .with_attributes(vec![("class", "header")])
            .with_container(Container::new(ContainerType::Div).with_header_attr(
                1,
                main_heading,
                vec![("class", "main-title bebas-neue-regular")],
            )),
    );

    let nav_container = HtmlElement::new(HtmlTag::Div)
        .with_attribute("class", "nav-container flex-column")
        .with_container(
            Container::new(ContainerType::Div)
                .with_attributes(vec![("class", "your-hours-div nav-div")])
                .with_link("#hours", "Your Hours"),
        )
        .with_container(
            Container::new(ContainerType::Div)
                .with_attributes(vec![("class", "date-and-times-div nav-div")])
                .with_link("#dates", "Date And Times"),
        );

    page.add_container(
        Container::new(ContainerType::Nav)
            .with_attributes(vec![("class", "nav oswald-font-500")])
            .with_html(nav_container),
    );

    let mut hours_div =
        HtmlElement::new(HtmlTag::Div).with_attribute("class", "hours-div flex-column adaptive");

    let mut dates_div = HtmlElement::new(HtmlTag::Div).with_attribute(
        "class",
        "dates-div flex-column flex-align-justify-center adaptive",
    );

    for line in hrs_lines {
        hours_div.add_paragraph(line);
    }

    date_lines.reverse();

    let mut counter: usize = 0;

    if date_lines.len() >= 7 {
        while counter <= 6 {
            dates_div.add_paragraph(date_lines[counter]);

            counter += 1;
        }
    } else {
        for line in date_lines {
            dates_div.add_paragraph(line);
        }
    }

    let hours_div_container = HtmlElement::new(HtmlTag::Div)
        .with_attribute("id", "hours")
        .with_attribute("class", "hours-div-container color flex-column")
        .with_header(2, "Your Hours Played")
        .with_html(hours_div);

    let dates_div_container = HtmlElement::new(HtmlTag::Div)
        .with_attribute("id", "dates")
        .with_attribute("class", "dates-div-container color flex-column")
        .with_header(2, "Your time played<br>in the last 7 sessions")
        .with_html(dates_div);

    page.add_container(
        Container::new(ContainerType::Main)
            .with_attributes(vec![("class", "main flex-column color oswald-font-500")])
            .with_html(hours_div_container)
            .with_html(dates_div_container),
    );

    page.add_container(
        Container::new(ContainerType::Footer)
            .with_attributes(vec![("class", "footer flex-row oswald-font-700")])
            .with_paragraph("&copy; OneilNvM 2024 ")
            .with_link_attr(
                "https://github.com/OneilNvM/rl-hours-tracker",
                "Rocket League Hours Tracker Github",
                [("target", "_blank")],
            ),
    );

    Ok(page.to_html_string())
}
