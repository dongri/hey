extern crate serde;
extern crate toml;

use serde::{Deserialize, Serialize};
use std::fs::{self};

use slack_hook::{PayloadBuilder, Slack};

use async_std::task;
use futures::executor::block_on;
use std::time::Duration;

use reqwest::ClientBuilder;
use reqwest::StatusCode;

use chrono::{DateTime, Local};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Server {
    name: String,
    url: String,
    method: String,
    timeout: u64,
    status_code: u16,
    slack_webhook: String,
    slack_channel_alert: String,
    slack_channel_log: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    interval: u64,
    servers: Vec<Server>,
}

#[tokio::main]
async fn main() {
    let server_toml: String = fs::read_to_string("Config.toml").unwrap();
    let config: Result<Config, toml::de::Error> = toml::from_str(&server_toml);
    match config {
        Ok(c) => loop {
            task::sleep(Duration::from_secs(c.interval)).await;
            let config = c.to_owned();
            let future = watcher(config);
            block_on(future);
        },
        Err(e) => panic!("Filed to parse TOML: {}", e),
    }
}

async fn watch_task(server: Server) {
    let local_datetime: DateTime<Local> = Local::now();
    let result = server_status(&server, local_datetime).await;
    match result {
        Ok(()) => {}
        Err(e) => {
            let text = make_message(false, &server, format!("{:?}", e), local_datetime);
            notify_to_slack(&server.slack_channel_alert, &server.slack_webhook, text);
        }
    }
}

async fn server_status(
    server: &Server,
    local_datetime: DateTime<Local>,
) -> Result<(), reqwest::Error> {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(server.timeout))
        .build()?;
    let res;
    match server.method.as_str() {
        "GET" => {
            res = client.get(&server.url).send().await?;
        }
        "POST" => {
            res = client.post(&server.url).send().await?;
        }
        _ => {
            res = client.get(&server.url).send().await?;
        }
    }
    let status_code = res.status();
    let ok = StatusCode::from_u16(server.status_code).unwrap();
    if status_code == ok {
        let text = make_message(true, server, format!("{}", ok), local_datetime);
        notify_to_slack(&server.slack_channel_log, &server.slack_webhook, text);
    } else {
        let text = make_message(false, server, format!("{}", status_code), local_datetime);
        notify_to_slack(&server.slack_channel_alert, &server.slack_webhook, text)
    };
    Ok(())
}

async fn watcher(c: Config) {
    let mut tasks = Vec::new();
    for server in c.servers {
        let task = watch_task(server);
        tasks.push(task);
    }
    futures::future::join_all(tasks).await;
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn make_message(
    is_ok: bool,
    server: &Server,
    status: String,
    local_datetime: DateTime<Local>,
) -> String {
    let mut at_channel: String = String::from("");
    if !is_ok {
        at_channel = String::from("@channel\n");
    }
    let text = format!(
        "{}```\n{}: {}\nStatus: {}\n{}\n```",
        at_channel, server.name, server.url, status, local_datetime
    );
    text
}

fn notify_to_slack(channel: &String, slack_webhook: &String, text: String) {
    let slack = Slack::new(string_to_static_str(slack_webhook.to_string())).unwrap();
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
        Err(e) => println!("ERR: {:?}", e),
    }
}
