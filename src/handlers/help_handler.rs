use crate::config::{Config, Storage};

use std::convert::From;

use ruma_client::{
    api::r0::message::create_message_event,
    events::{
        room::message::{MessageEventContent, NoticeMessageEventContent, TextMessageEventContent},
        EventJson, EventType,
    },
    identifiers::RoomId,
    HttpsClient,
};
use slog::{debug, error, trace, Logger};

#[derive(Debug)]
enum Command {
    ActionCommand,
    ActionCommandless,
    GroupPing,
    GithubSearch,
    Link,
    UnitConversion,
    UnknownCommand,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_ref() {
            "command" => Command::ActionCommand,
            "commandless" => Command::ActionCommandless,
            "ping" => Command::GroupPing,
            "github-search" => Command::GithubSearch,
            "link" => Command::Link,
            "unit-conversion" => Command::UnitConversion,
            _ => Command::UnknownCommand,
        }
    }
}

pub(super) async fn help_handler(
    text: &TextMessageEventContent,
    room_id: &RoomId,
    client: &HttpsClient,
    mut storage: &mut Storage,
    config: &Config,
    logger: &Logger,
) {
    if config.help_rooms.is_empty() || config.help_rooms.contains(room_id) {
        trace!(logger, "Room is allowed, building help message");
        let mut message = String::new();
        match text.body.split(' ').nth(1).map(|x| Command::from(x)) {
            Some(v) => match v {
                Command::ActionCommand => message = action_command_help_message().await,
                Command::ActionCommandless => message = action_commandless_help_message().await,
                Command::GroupPing => message = group_ping_help_message(&config).await,
                Command::GithubSearch => message = github_search_help_message(&config).await,
                Command::Link => message = link_help_message(&config).await,
                Command::UnitConversion => message = unit_conversion_help_message(&config).await,
                Command::UnknownCommand => (),
            },
            None => {
                trace!(logger, "Printing help message for program");
                message = generic_help_message().await;
            }
        };
        if !message.is_empty() {
            send_help_message(&room_id, &client, &mut storage, message, &logger).await;
        } else {
            debug!(logger, "Unknown command");
        }
    } else {
        trace!(
            logger,
            "Rooms are limited and room {} is not in the allowed list of help command rooms",
            room_id
        );
    }
}

async fn generic_help_message() -> String {
    format!("Matrix Bot v{}
Repository: {}

This bot has two types of actions it can perform: command and commandless
Use the !help command to learn more about their characteristics

USAGE:
\t!help command|commandless
\t!help [ACTION]

ACTION TYPES:
\tcommand\t\tCommand actions are a message that starts with !
\tcommandless\tCommandless actions are any message that meets the critera to trigger an action and do not start with an !

ACTIONS:
\tping\t\t\tPing a group of people
\tgithub-search\tSearch github by project and issue/PR number
\tlink\t\t\t\tShortcuts for linking helpful URLs
\tunit-conversion\tConvert common conversational units",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY")
    )
}

async fn action_command_help_message() -> String {
    format!("Command Action

Command actions are defined as message that have no formatting (like no italics, no inline code, not a reply, etc) that start with a !. These can only perform one action per message.

EXAMPLES:
\t!help
\t!convert 22mi")
}

async fn action_commandless_help_message() -> String {
    format!("Commandless Action
    
Commandles actions can happen in any plain text message but certain text formatting will be ignored. Currently ignored formatting is inline code, code blocks, and the text in a reply (but not the reply itself)    

The exact rules for triggering a commandless action vary by action (so check action help pages for info on how to trigger them), but their defining features are that they can be in any part of a message and mutiple can be triggered per message.

EXAMPLES:
\tHey there, i think you want to read docs@troubleshooting
\tIts not like 32f is that cold. not sure what you are complaining about
")
}

async fn group_ping_help_message(config: &Config) -> String {
    let mut groups = Vec::new();
    for (group, _) in &config.group_pings {
        groups.push(group);
    }
    groups.sort();
    let mut available_groups = String::new();
    for group in groups {
        available_groups.push_str(group);
        available_groups.push('|');
    }
    available_groups.pop();
    let available_groups = available_groups.replace('|', " | ");
    format!("Group Ping

This action is only available as commandless. It will trigger on anything that matches \"%group\" where \"group\" is the group you want to ping.

If the group exists and you are authorized to make a group ping, a message pinging everyone in the group will be made in a bot message.

USAGE:
\tHey there %server can you look at this for me?
\t%server

AVAILABLE GROUPS:
{}", available_groups
    )
}

async fn github_search_help_message(config: &Config) -> String {
    let mut repos = Vec::new();
    for (repo, _) in &config.repos {
        repos.push(repo);
    }
    repos.sort();
    let mut available_repos = String::new();
    for repo in repos {
        available_repos.push_str(repo);
        available_repos.push('|');
    }
    available_repos.pop();
    let available_repos = available_repos.replace('|', " | ");
    format!("Github Search

This action is only available as commandless. It will trigger on anything that matches \"jf#1234\" where \"jf\" is the repo you want to search and \"1234\" is the issue or PR you want to link.

If the repo and the number exist, it will provide a link to the issue or pull in a bot message.

USAGE: 
\tI could use a review on jf#1234
\tjf#1234

AVAILABLE REPOS:
{}", available_repos)
}

async fn link_help_message(config: &Config) -> String {
    let mut keywords = Vec::new();
    for keyword in &config.linkers {
        keywords.push(keyword);
    }
    keywords.sort();
    let mut available_keywords = String::new();
    for keyword in keywords {
        available_keywords.push_str(&keyword);
        available_keywords.push('|');
    }
    available_keywords.pop();
    let available_keywords = available_keywords.replace('|', " | ");
    let mut links = Vec::new();
    for (link, _) in &config.links {
        links.push(link);
    }
    links.sort();
    let mut available_links = String::new();
    for link in links {
        available_links.push_str(&link);
        available_links.push('|');
    }
    available_links.pop();
    let available_links = available_links.replace('|', " | ");
    format!("Link

This action is only available as commandless. It will trigger on anything that matches \"link@hwa\" where \"link\" is a configured keyword and \"hwa\" is a linkable item.

if the keyword and item exist, there will be a link provided in a bot message.

USAGE:
\tI think you might want to look at link@hwa
\tlink@hwa

AVAILABLE KEYWORDS:
{}

AVAILABLE LINKS:
{}
    ", available_keywords, available_links)
}

async fn unit_conversion_help_message(config: &Config) -> String {
    let mut units = Vec::new();
    for unit in &config.unit_conversion_exclusion {
        units.push(unit);
    }
    units.sort();
    let mut space_excluded_units = String::new();
    for unit in units {
        space_excluded_units.push_str(&unit);
        space_excluded_units.push('|');
    }
    space_excluded_units.pop();
    let space_excluded_units = space_excluded_units.replace('|', " | ");
    format!("Unit Conversion

This action is available as both a command and commanless. It will convert common converstation units Imperial <-> Metric to help ease international chat. There can be a space between the quantity and unit except for the units excluded by configuration (listed below).

USAGE:
\tCOMMAND:
\t\t!convert 20c

\tCOMMANDLESS:
\t\tIt's weird that the speed limit here is 45mph
\t\t45 mph

SUPPORTED UNITS:
LENGTH:
cm | m | km | in | ft | mi | mile | miles
TEMPERATURE:
c | °c | f | °f
WEIGHT:
kg | lbs
SPEED:
km/h | kmh | kph | kmph | mph

SPACE EXCLUDED UNITS:
{}
    ", space_excluded_units)
}

async fn send_help_message(
    room_id: &RoomId,
    client: &HttpsClient,
    storage: &mut Storage,
    message: String,
    logger: &Logger,
) {
    match client
        .request(create_message_event::Request {
            room_id: room_id.clone(), //FIXME: There has to be a better way than to clone here
            event_type: EventType::RoomMessage,
            txn_id: storage.next_txn_id(),
            data: EventJson::from(MessageEventContent::Notice(NoticeMessageEventContent {
                body: message,
                relates_to: None,
                format: None,
                formatted_body: None,
            }))
            .into_json(),
        })
        .await
    {
        Ok(_) => (),
        Err(e) => error!(logger, "{:?}", e),
    }
}
