extern crate reqwest;
extern crate scraper;
extern crate urlencoding;
extern crate rpassword;

mod gradescope;

use std::{process::exit, io::{self, Write}};
use gradescope::client::{GradescopeClient, ClientError};

fn main() {
    /* Create a client which will perform all communications with Gradescope */
    let mut client = GradescopeClient::new().unwrap();

    /* TODO: check for saved creds */

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
        email
    };

    /* Get password */
    let password = match rpassword::prompt_password_stdout("Enter your Gradescope password: ") {
        Ok(password) => password,
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
        _ => {
            eprintln!("There was an error, please try again.");
            exit(1);
        }
    }

//    client.submit_files(171498, 702483, vec!["src/main.rs"]).unwrap();
}
