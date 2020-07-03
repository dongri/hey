extern crate serde;
extern crate toml;

use serde::{Deserialize, Serialize};
use std::fs::{self};

use slack_hook::{Slack, PayloadBuilder};

use std::time::Duration;
use futures::executor::block_on;
use async_std::task;

use reqwest::StatusCode;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Server {
    name: String,
    url: String,
    status_code: u16,
    slack_webhook: String,
    slack_channel_alert: String,
    slack_channel_log: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Servers {
    servers: Vec<Server>
}

#[tokio::main]
async fn main() {
    loop {
        task::sleep(Duration::from_secs(60)).await;
        let future = async_request();
        block_on(future);
    }
}

async fn check(server: Server) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let target_server = server.to_owned();

    let res = client.get(&target_server.url).send().await.unwrap();
    let status_code = res.status();
    let ok = StatusCode::from_u16(target_server.status_code).unwrap();
    if status_code == ok {
        let text = format!("{}: {}\nStatus: {}", target_server.name, target_server.url, ok);
        notify_to_slack(target_server.slack_channel_log, target_server.slack_webhook, text.to_string());
    } else {
        let text = format!("@channel\n{}: {}\nStatus: {}", target_server.name, target_server.url, status_code);
        notify_to_slack(target_server.slack_channel_alert, target_server.slack_webhook, text.to_string())
    };
}

async fn async_request() {
    let server_toml: String = fs::read_to_string("Servers.toml").unwrap();
    let servers: Result<Servers, toml::de::Error> = toml::from_str(&server_toml);

    match servers {
        Ok(p) => {
            let mut tasks = Vec::new();
            for server in p.servers {
                let task = check(server);
                tasks.push(task);
            }
            futures::future::join_all(tasks).await;
        }
        Err(e) => panic!("Filed to parse TOML: {}", e),
    }

}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn notify_to_slack(channel: String, slack_webhook: String, text: String) {
    let slack = Slack::new(string_to_static_str(slack_webhook)).unwrap();
    let pb = PayloadBuilder::new()
        .text(text)
        .channel(channel)
        .username("Hey")
        .icon_emoji(":chart_with_upwards_trend:")
        .link_names(true)
        .build()
        .unwrap();

    let slack_res = slack.send(&pb);
    match slack_res {
        Ok(()) => println!("ok"),
        Err(e) => println!("ERR: {:?}",e)
    }
}