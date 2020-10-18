extern crate reqwest;
extern crate scraper;
extern crate urlencoding;

mod gradescope;

use gradescope::client::GradescopeClient;

fn main() {
    let mut client = GradescopeClient::new().unwrap();
    client.login("vinnie@vcaprarola.me".into(), "".into()).unwrap();
    client.submit_files(171498, 702483, vec!["src/main.rs"]).unwrap();
}
