extern crate reqwest;
extern crate scraper;
extern crate urlencoding;
extern crate rpassword;
extern crate toml;
extern crate serde;
extern crate dirs;

mod gradescope;
mod config;

use std::{process::exit, io::{self, Write}, fs, path::PathBuf};
use config::Config;
use gradescope::client::{GradescopeClient, ClientError};

fn get_config_dir() -> Option<PathBuf> {
    /* Get path for config folder */
    let mut config = match dirs::home_dir() {
        Some(d) => d,
        None => return None
    };

    /* Append gradescope config folder */
    config.push(".gradescope-submit");

    /* Try to create the folder */
    match fs::create_dir(&config) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(_) => return None,
    }

    Some(config)
}

fn save_token(token: String) {
    /* Get path for config folder */
    if let Some(mut token_file) = get_config_dir() {
        /* Append token filename */
        token_file.push("signed_token");

        /* Write the token to the file */
        fs::write(token_file, token).ok();
    }
}

fn load_cookie_from_file() -> Option<String> {
    /* Get path for config folder */
    if let Some(mut token_file) = get_config_dir() {
        /* Append token filename */
        token_file.push("signed_token");

        /* Read the token from the file */
        match fs::read_to_string(token_file) {
            Ok(token) => Some(token),
            Err(_) => None,
        }
    } else {
        None
    }
}

fn login_prompt(client: &mut GradescopeClient) {
    /* Get user's email */
    let email = {
        /* Display a prompt (need to flush b/c no newline) */
        print!("Enter your Gradescope email: ");
        io::stdout().flush().unwrap();

        /* Get input */
        let mut email = String::new();
        match io::stdin().read_line(&mut email) {
            Ok(0) => {
                eprintln!("No email entered");
                exit(1);
            }
            Err(_) => {
                eprintln!("Error reading input");
                exit(1);
            }
            _ => ()
        }
        
        /* Remove the newline */
        if email.chars().last() == Some('\n') {
            email.pop();
        }

        email
    };

    /* Get password */
    let password = match rpassword::prompt_password_stdout("Enter your Gradescope password: ") {
        Ok(password) => {
            println!("");
            password
        },
        Err(_) => {
            eprintln!("Error reading input");
            exit(1);
        }
    };

    match client.login(email, password) {
        Err(ClientError::InvalidLogin) => {
            eprintln!("It looks like you entered an incorrect email and/or password!");
            exit(1);
        }
        Err(_) => {
            eprintln!("There was an error, please try again.");
            exit(1);
        }
        Ok(token) => {
            save_token(token);
        }
    }
}

fn main() {
    /* Parse the .submit file */
    let config = match Config::load(".submit") {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Could not read .submit file.");
            exit(1)
        }
    };

    /* Create a client which will perform all communications with Gradescope */
    let client = {
        /* Initialize client */
        let mut client = GradescopeClient::new(load_cookie_from_file()).unwrap();

        /* If the client is not logged in, request username and password */
        if !client.is_logged_in() {
            login_prompt(&mut client);
        }

        client
    };

    client.submit_files(config.course.id, config.assignment.id, config.assignment.files).unwrap();
}
