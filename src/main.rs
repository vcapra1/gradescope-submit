extern crate reqwest;
extern crate scraper;
extern crate urlencoding;
extern crate rpassword;
extern crate toml;
extern crate serde;

mod gradescope;
mod config;

use std::{process::exit, io::{self, Write}};
use config::Config;
use gradescope::client::{GradescopeClient, ClientError};

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
        /* Get user's email */
        let email = {
            /* TODO: check for saved session cookie */

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

        let mut client = GradescopeClient::new().unwrap();

        match client.login(email, password) {
            Err(ClientError::InvalidLogin) => {
                eprintln!("It looks like you entered an incorrect email and/or password!");
                exit(1);
            }
            Err(_) => {
                eprintln!("There was an error, please try again.");
                exit(1);
            }
            _ => ()
        }

        client
    };

    client.submit_files(config.course.id, config.assignment.id, config.assignment.files).unwrap();
}
