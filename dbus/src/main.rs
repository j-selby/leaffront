extern crate notify_rust;
extern crate redis;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use notify_rust::server::NotificationServer;

#[derive(Serialize, Deserialize)]
pub struct Notification {
    name: String,
    contents: String,
}

fn main() {
    let mut server = NotificationServer::create();

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let sub = client.get_connection().unwrap();

    println!("Ready!");

    NotificationServer::start(&server, |ref notify| {
        let notification = Notification {
            name: notify.appname.to_owned(),
            contents: notify.body.to_owned(),
        };
        let v: String = serde_json::to_string(&notification).unwrap();
        let k: &str = "leaffront.notify";

        redis::cmd("PUBLISH").arg(k).arg(&v).execute(&sub);
    });
}
