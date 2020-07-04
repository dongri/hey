# Hey

HTTP health checker.

### Edit Config.toml
```toml
interval = 600

[[servers]]
name = "staing api"
url = "https://api.staging.example.com/status"
status_code = 200
slack_webhook = "https://hooks.slack.com/services/***/***"
slack_channel_alert = "staging-alert"
slack_channel_log = "staging-log"

[[servers]]
name = "production api"
url = "https://api.example.com/status"
status_code = 200
slack_webhook = "https://hooks.slack.com/services/***/***"
slack_channel_alert = "production-alert"
slack_channel_log = "production-log"
```

### Run on docker-compose
```sh
$ vim Config.toml

$ docker-compose up --build -d
```

### Run on Linux
```sh
$ vim Config.toml

$ ./run.sh
```

### Slack

![Hey](https://raw.githubusercontent.com/dongri/images/master/hey-alert.png)


# Roadmap
- [ ] Supoort POST method
- [ ] Suppoert basic auth
