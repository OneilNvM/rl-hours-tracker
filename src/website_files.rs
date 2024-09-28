//! This module contains the functionality to generate the Html, CSS, and JavaScript for the
//! Rocket League Hours Tracker website.
use build_html::{Container, ContainerType, Html, HtmlContainer, HtmlPage};
use build_html::{HtmlElement, HtmlTag};
use std::io;
use std::io::{Error, ErrorKind};
use std::{
    fs::{write, File},
    io::{Read, Write},
};
use webbrowser;
use tokio::runtime::Runtime;
use reqwest::{Client, Response};

pub struct Github<'a> {
    owner: &'a str,
    repo: &'a str,
    branch: &'a str,
    path: &'a str,
    file: &'a str,
    url: String,
}

impl<'a> Github<'a> {
    pub fn new() -> Github<'a> {
        Github {
            owner: "",
            repo: "",
            branch: "",
            path: "",
            file: "",
            url: String::new(),
        }
    }

    pub fn set_params(&mut self, owner: &'a str, repo: &'a str, branch: &'a str, path: &'a str, file: &'a str) {
        self.owner = owner;
        self.repo = repo;
        self.branch = branch;
        self.path = path;
        self.file = file;
    }

    pub fn build_url(&mut self) {
        let url = format!("https://raw.githubusercontent.com/{}/{}/refs/heads/{}/{}/{}", self.owner, self.repo, self.branch, self.path, self.file);
        self.url = url;
    }
}

async fn send_request(url: String) -> Response {
    let client = Client::new();

    let request = client.get(url).send().await;

    match request {
        Ok(response) => response,
        Err(e) => panic!("There was an issue requesting a website file.\n{:?}", e.status()),
    }
}

async fn handle_response(urls: Vec<String>) -> Vec<String> {
    let mut text_vec: Vec<String> = Vec::new();

    for url in urls {
        let response = send_request(url).await;

        let text = response.text().await;

        match text {
            Ok(result) => text_vec.push(result),
            Err(e) => panic!("There was an issue when retrieving full response text.\n{e}")
        }
    }

    text_vec
}

fn run_async_functions(urls: Vec<String>) -> Vec<String> {
    let rt = Runtime::new().unwrap();

    let result = rt.block_on(handle_response(urls));

    result
}

/// This function is used to generate the necessary files for the Rocket League Hours Tracker website.
/// It accepts a bool [`bool`] as an argument which determines whether the option to open the website
/// in the browser should appear or not.
pub fn generate_website_files(boolean: bool) {
    // Create and open files
    let index = File::create("C:\\RLHoursFolder\\website\\pages\\index.html");
    let main_styles = File::create("C:\\RLHoursFolder\\website\\css\\main.css");
    let home_styles = File::create("C:\\RLHoursFolder\\website\\css\\home.css");
    let animations_js = File::create("C:\\RLHoursFolder\\website\\js\\animations.js");
    let mut hours_file = File::open("C:\\RLHoursFolder\\hours.txt");
    let mut date_file = File::open("C:\\RLHoursFolder\\date.txt");

    let mut github_main_css = Github::new();
    let mut github_home_css = Github::new();
    let mut github_animations_js = Github::new();
    let mut github_grey_icon = Github::new();
    let mut github_white_icon = Github::new();

    github_main_css.set_params("OneilNvM", "rl-hours-tracker", "github-http", "website/css", "main.css");
    github_home_css.set_params("OneilNvM", "rl-hours-tracker", "github-http", "website/css", "home.css");
    github_animations_js.set_params("OneilNvM", "rl-hours-tracker", "github-http", "website/js", "animations.js");
    github_grey_icon.set_params("OneilNvM", "rl-hours-tracker", "github-http", "website/images", "rl-icon-grey.png");
    github_white_icon.set_params("OneilNvM", "rl-hours-tracker", "github-http", "website/images", "rl-icon-white.png");

    let github_vec = vec![github_main_css.url, github_home_css.url, github_animations_js.url];

    let responses = run_async_functions(github_vec);

    let _ = write("C:\\RLHoursFolder\\website\\images\\rl-icon-grey.png", responses[3].clone());
    let _ = write("C:\\RLHoursFolder\\website\\images\\rl-icon-white.png", responses[4].clone());

    // Creates the main.css file 
    match main_styles {
        Ok(mut ms_file) => {
            // Writes the CSS content to the file
            match ms_file.write_all(responses[0].as_bytes()) {
                Ok(_) => (),
                Err(e) => panic!("There was an issue when writing to main.css: {e}")
            }
        }
        Err(e) => panic!("There was an issue with main styles: {:?}", e),
    }

    // Creates the home.css file
    match home_styles {
        Ok(mut hs_file) => {
            // Writes the CSS content to the file
            match hs_file.write_all(responses[1].as_bytes()) {
                Ok(_) => (),
                Err(e) => panic!("There was an issue when writing to home.css: {e}")
            }
        }
        Err(e) => panic!("There was an issue when creating main.css: {e}"),
    }

    // Creates the animations.js file
    match animations_js {
        Ok(mut a_js_file) => {
            // Writes the JavaScript content to the file
            match a_js_file.write_all(responses[2].as_bytes()) {
                Ok(_) => (),
                Err(e) => panic!("There was an issue when writing to the animations JavaScript file: {e}")
            }
        }
        Err(e) => panic!("There was an issue when creating the animations JavaScript file: {e}"),
    }

    // Creates the index.html file
    match index {
        Ok(mut idx_file) => {
            // Declare uninitialized 'contents' string variable to store the Html for the website
            let contents: String;

            // Generate the website and handle any errors
            match generate_page(&mut hours_file, &mut date_file) {
                Ok(page) => {
                    // Initialize the 'contents' variable with the Html
                    contents = page.replace("<body>", "<body class=\"body adaptive\">");
                }
                Err(e) => {
                    println!("Error in 'generate_page', website not generated. Error Kind: {}\nError message: {e}", e.kind());
                    return;
                }
            }

            // Writes the index.html file
            match idx_file.write_all(&contents.as_bytes()) {
                Ok(_) => {
                    // If statement determines whether to prompt the user with the option to open the website
                    if boolean == true {
                        let mut option = String::new();

                        println!("Open hours website in browser (y/n)?");
                        io::stdin().read_line(&mut option).unwrap();

                        if option.trim() == "y" || option.trim() == "Y" {
                            if webbrowser::open("C:\\RLHoursFolder\\website\\pages\\index.html").is_ok() {
                                println!("200 OK");
                            };
                        }
                    }
                }
                Err(e) => panic!("There was an issue when writing index.html: {e}"),
            }
        }
        Err(e) => panic!("There was an issue when creating index.html: {e}"),
    }
}

/// This function generates the necessary Html for the website via the [`build_html`] library. The `hours_file` and `date_file`
/// parameters are both mutable [`Result<File>`] references which provides us with a [`File`] if it is successful, or [`io::Error`] if
/// it fails. This function then returns a [`Result<String>`] of the Html.
/// 
/// # Errors
/// This function returns an [`io::Error`] if there were any errors during file operations.
fn generate_page(
    hours_file: &mut Result<File, io::Error>,
    date_file: &mut Result<File, io::Error>,
) -> Result<String, io::Error> {
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
        if let Ok(_) = hrs_file.read_to_string(&mut hrs_content) {
            ();
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "The files contents are not valid UTF-8.",
            ));
        }
    } else {
        return Err(Error::new(ErrorKind::NotFound, "The file 'hours.txt' could not be opened. Either it does not exist or it is not in the 'RLHoursFolder' directory."));
    }

    if let Ok(ref mut dt_file) = date_file {
        if let Ok(_) = dt_file.read_to_string(&mut date_content) {
            ();
        } else {
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
            .with_paragraph("OneilNvM 2024 &copy;")
            .with_link_attr(
                "https://github.com/OneilNvM/rl-hours-tracker",
                "Rocket League Hours Tracker Github",
                [("target", "_blank")],
            ),
    );

    Ok(page.to_html_string())
}
