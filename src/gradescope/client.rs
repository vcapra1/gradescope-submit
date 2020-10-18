use std::path::Path;
use reqwest::{redirect::Policy, blocking::{Client, multipart::{Form, Part}}};
use scraper::{html::Html, selector::Selector};
use urlencoding::encode;

pub struct GradescopeClient {
    http_client: Client,
    logged_in: bool,
}

#[derive(Debug)]
pub enum ClientError {
    InitError,
    HttpError,
    UnexpectedResponse,
    InvalidLogin,
    InvalidState,
    FileError(String),
}

impl GradescopeClient {
    /// Initiailizes a new `GradescopeClient`, not yet logged in, so the login method must be used
    /// before doing anything else.
    ///
    /// [`GradescopeClient`]: struct.GradescopeClient.html
    ///
    /// # Errors
    ///
    /// Returns [`InitError`] if there is an error setting up the client.
    ///
    /// [`InitError`]: enum.ClientError.html
    ///
    pub fn new() -> Result<Self, ClientError> {
        /* Create a new http client */
        let builder = Client::builder()
            .redirect(Policy::none())
            .cookie_store(true);

        /* Build the client */
        let http_client = match builder.build() {
            Ok(client) => Ok(client),
            Err(_) => Err(ClientError::InitError),
        }?;

        Ok(GradescopeClient {
            http_client,
            logged_in: false,
        })
    }

    /// Given the email and password of a user, attempts to log in.
    ///
    /// # Arguments
    ///
    /// * `email` - The email associated with the user's [Gradescope] account
    /// * `password` - The password for the user's [Gradescope] account
    ///
    /// # Errors
    ///
    /// If there is an error while communicating with [Gradescope], [`HttpError`] will be returned.  If
    /// the user's credentials are incorrect, [`InvalidLogin`] will be returned.  If a response is
    /// received that cannot be parsed, [`UnexpectedResponse`] will be returned.
    ///
    /// [Gradescope]: https://www.gradescope.com/
    /// [`InvalidLogin`]: enum.ClientError.html
    /// [`HttpError`]: enum.ClientError.html
    /// [`UnexpectedResponse`]: enum.ClientError.html
    ///
    pub fn login(&mut self, email: String, password: String) -> Result<(), ClientError> {
        /* Acquire a CSRF token */
        let csrf_token = {
            /* Make the initial request to the login page */
            let response = match self.http_client.get("https://www.gradescope.com/login").send() {
                Ok(response) => Ok(response),
                Err(_) => Err(ClientError::HttpError),
            }?;

            /* Check the status of the response and extract the HTML document */
            let response_body = if response.status() == 200 {
                match response.text() {
                    Ok(text) => Ok(text),
                    Err(_) => Err(ClientError::HttpError)
                }
            } else {
                Err(ClientError::HttpError)
            }?;

            /* Parse the response */
            let document = Html::parse_document(&response_body);

            /* Find the token input element */
            let token_element = {
                /* Create a selector */
                let selector = Selector::parse("input[name=authenticity_token]").unwrap();

                /* Find the element */
                match document.select(&selector).next() {
                    Some(elem) => Ok(elem),
                    None => Err(ClientError::UnexpectedResponse),
                }
            }?.value();

            /* Get the value attribute */
            if let Some(token) = token_element.attr("value") {
                Ok(String::from(token))
            } else {
                Err(ClientError::UnexpectedResponse)
            }?
        };

        /* Make the login request with the credentials and token */
        let login_successful = {
            /* Construct the body of the request */
            let request_body = format!("authenticity_token={}&session[email]={}\
                &session[password]={}&session[remember_me]=0&commit=Log in\
                &session[remember_me_sso]=0",
                encode(&csrf_token), encode(&email), encode(&password));

            /* Build the request */
            let request = self.http_client.post("https://www.gradescope.com/login")
                .body(request_body)
                .header("Host", "www.gradescope.com")
                .header("Referer", "https://www.gradescope.com");

            /* Make a POST request to the login page */
            let response = match request.send() {
                Ok(response) => Ok(response),
                Err(_) => Err(ClientError::HttpError),
            }?;

            response.status() == 302
/*
            /* Check the status of the response and extract the HTML document */
            let response_body = if response.status() == 200 {
                match response.text() {
                    Ok(text) => Ok(text),
                    Err(_) => Err(ClientError::HttpError)
                }
            } else {
                Err(ClientError::HttpError)
            }?;

            /* Parse the response */
            let document = Html::parse_document(&response_body);

            /* Get the page title */
            let title = {
                /* Create a selector */
                let selector = Selector::parse("title").unwrap();

                /* Find the element */
                match document.select(&selector).next() {
                    Some(elem) => Ok(elem),
                    None => Err(ClientError::UnexpectedResponse),
                }
            }?.inner_html();

            /* Make sure login was successful */
            title == "Dashboard | Gradescope"*/
        };

        if login_successful {
            /* Set logged in flag */
            self.logged_in = true;

            Ok(())
        } else {
            Err(ClientError::InvalidLogin)
        }
    }

    /// Submit a set of files from the local machine to the [Gradescope] assignment with the
    /// specified ID.
    ///
    /// # Arguments
    ///
    /// * `course_id` - The course ID on [Gradsecope]
    /// * `assignment_id` - The ID of the assignment to submit to
    /// * `files` - A list of filenames to submit
    ///
    /// [Gradescope]: https://www.gradescope.com/
    ///
    /// # Errors
    ///
    /// If there is an error while communicating with [Gradescope], [`HttpError`] will be returned.
    /// If the user is not logged in, [`InvalidState`] will be returned.  If a response is received
    /// that cannot be parsed, [`UnexpectedResponse`] will be returned.
    ///
    /// [Gradescope]: https://www.gradescope.com/
    /// [`InvalidState`]: enum.ClientError.html
    /// [`HttpError`]: enum.ClientError.html
    /// [`UnexpectedResponse`]: enum.ClientError.html
    ///
    pub fn submit_files<T: AsRef<Path>>(&self,
                                        course_id: u64,
                                        assignment_id: u64,
                                        files: Vec<T>) -> Result<(), ClientError> {
        /* Make sure we are logged in */
        if !self.logged_in {
            return Err(ClientError::InvalidState);
        }

        /* Acquire a CSRF token */
        let csrf_token = {
            /* Construct the URL */
            let url = format!("https://www.gradescope.com/courses/{}", course_id);

            /* Make the initial request to the login page */
            let response = match self.http_client.get(&url).send() {
                Ok(response) => Ok(response),
                Err(_) => Err(ClientError::HttpError),
            }?;

            /* Check the status of the response and extract the HTML document */
            let response_body = if response.status() == 200 {
                match response.text() {
                    Ok(text) => Ok(text),
                    Err(_) => Err(ClientError::HttpError)
                }
            } else {
                Err(ClientError::HttpError)
            }?;

            /* Parse the response */
            let document = Html::parse_document(&response_body);

            /* Find the token input element */
            let token_element = {
                /* Create a selector */
                let selector = Selector::parse("meta[name=csrf-token]").unwrap();

                /* Find the element */
                match document.select(&selector).next() {
                    Some(elem) => Ok(elem),
                    None => Err(ClientError::UnexpectedResponse),
                }
            }?.value();

            /* Get the value attribute */
            if let Some(token) = token_element.attr("content") {
                Ok(String::from(token))
            } else {
                Err(ClientError::UnexpectedResponse)
            }?
        };

        /* Submit the files to the project page */
        {
            /* Construct the URL */
            let url = format!("https://www.gradescope.com/courses/{}/assignments/{}/submissions", course_id, assignment_id);

            /* Build the multipart form with all the file */
            let form = {
                /* Create a new form and add text fields to it */
                let form = Form::new()
                    .text("authenticity_token", csrf_token)
                    .text("submission[method]", "uplaod");

                /* For each filename given, construct a part */
                files.into_iter().fold(Ok(form), |form, path| {
                    /* Get pathname */
                    let pathname = path.as_ref().display().to_string();

                    /* Get file name */
                    let filename = match path.as_ref().file_name() {
                        Some(filename) => Ok(String::from(filename.to_str().unwrap())),
                        None => Err(ClientError::FileError(pathname.clone())),
                    }?;

                    /* Create a new part */
                    let part = match Part::file(path) {
                        Ok(part) => Ok(part),
                        Err(_) => Err(ClientError::FileError(pathname.clone())),
                    }?.file_name(filename);

                    let part = match part.mime_str("application/octet-stream") {
                        Ok(part) => Ok(part),
                        Err(_) => Err(ClientError::FileError(pathname.clone())),
                    }?;

                    Ok(form?.part("submission[files][]", part))
                })?
            };

            /* Build the request */
            let request = self.http_client.post(&url).multipart(form);

            /* Send the request */
            let response = match request.send() {
                Ok(response) => Ok(response),
                Err(_) => Err(ClientError::HttpError),
            }?;

            println!("{}", response.text().unwrap());
            todo!();
        }
    }
}