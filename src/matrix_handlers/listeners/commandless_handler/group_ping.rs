//! Performs group pings based on message text and builds proper response

use crate::config::MatrixListenerConfig;
use crate::helpers::{clean_text, MatrixFormattedTextResponse};
use crate::regex::GROUP_PING;

use std::collections::HashSet;

use ruma_client::{events::room::message::TextMessageEventContent, identifiers::UserId};
use tracing::{debug, error, trace};

/// Finds requested users to ping and builds response text
pub fn group_ping(
    text: &TextMessageEventContent,
    sender: &UserId,
    config: &MatrixListenerConfig,
    text_response: &mut MatrixFormattedTextResponse,
) {
    let mut users: HashSet<UserId> = HashSet::new();
    if !config.group_ping_users.contains(sender) {
        debug!("User not authorized for group pings. Ignoring...");
        return;
    }
    match &text.formatted_body {
        Some(v) => {
            let clean_text = clean_text(v);
            if GROUP_PING.is_match(&clean_text) {
                for cap in GROUP_PING.captures_iter(&clean_text.to_lowercase()) {
                    trace!("{:?}", cap);
                    match config.group_pings.get(&cap[1]) {
                        Some(v) => {
                            for user in v {
                                users.insert(user.clone());
                            }
                        }
                        None => error!("Somehow lost group between regex match and insertion!"),
                    }
                }
            } else {
                debug!("There are no remaining matches after cleaning tags. Doing nothing.");
                return;
            }
        }
        None => {
            for cap in GROUP_PING.captures_iter(&text.body.to_lowercase()) {
                trace!("{:?}", cap);
                match config.group_pings.get(&cap[1]) {
                    Some(v) => {
                        for user in v {
                            users.insert(user.clone());
                        }
                    }
                    None => error!("Somehow lost group between regex match and insertion!"),
                }
            }
        }
    }
    if users.is_empty() {
        debug!("No users to ping after processing.");
    } else {
        text_response.set_users(users);
    }
}
