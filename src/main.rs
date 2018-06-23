extern crate irc;
extern crate cask;
extern crate regex;

use std::str;
use regex::Regex;
use irc::client::prelude::*;
use cask::CaskOptions;

fn main() {
    let cask = CaskOptions::default()
        .open("cask.db")
        .unwrap();


    let replies = CaskOptions::default()
        .open("replies.db")
        .unwrap();


    let config = Config::load("config.toml").unwrap();

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, move |client, message| {
        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if msg.starts_with(client.current_nickname()) {
                let re = Regex::new(&format!("^{}[^a-zA-Z0-9]*", client.current_nickname())).unwrap();
                let msg = re.replace(msg, "");

                if message_is_question(&msg) {
                    let key = get_key_from_question(&msg);
                    match cask.get(&key) {
                        Ok(v) => match v {
                            Some(value) => client.send_privmsg(target, &format!("{} is {}", key, str::from_utf8(&value).unwrap()))?,
                            None => client.send_privmsg(target, &format!("I don't know about {}", key))?,
                        },
                        Err(_) => client.send_privmsg(target, &format!("Something went wrong when looking up {}", key))?,
                    }
                } else if msg.contains(" is <reply> ") {
                    let mut splitter = msg.splitn(2, " is <reply> ");
                    let key = splitter.next().unwrap();
                    let value = splitter.next().unwrap();
                    replies.put(key.to_lowercase(), value).unwrap();
                } else if msg.contains(" is ") {
                    let mut splitter = msg.splitn(2, " is ");
                    let key = splitter.next().unwrap();
                    let value = splitter.next().unwrap();
                    cask.put(key.to_lowercase(), value).unwrap();
                } else {
                    let key = msg.trim().to_lowercase();
                    match replies.get(key) {
                        Ok(v) => match v {
                            Some(value) => client.send_privmsg(target, str::from_utf8(&value).unwrap())?,
                            None => {},
                        },
                        Err(_) => {},
                    }
                }
            }
        }
        Ok(())
    });

    reactor.run().unwrap();
}


fn message_is_question(msg: &str) -> bool {
    if msg.to_lowercase().starts_with("what") {
        return true;
    }

    return false;
}


fn get_key_from_question(msg: &str) -> String {
    let re = Regex::new(r"^what (is )?").unwrap();
    return re.replace(&msg.to_lowercase(), "").to_string();
}
