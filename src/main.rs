#[macro_use(handler)]
extern crate chatbot;
extern crate cask;

use std::str;

use chatbot::Chatbot;
use chatbot::adapter::{CliAdapter, IrcAdapter};

use cask::CaskOptions;

fn main() {
    let cask = CaskOptions::default()
        .open("cask.db")
        .unwrap();


    let replies = CaskOptions::default()
        .open("replies.db")
        .unwrap();



    let config = chatbot::adapter::IrcConfig::load("config.toml").unwrap();
    let name = config.clone().nickname.unwrap();
    let mut bot = Chatbot::new(&name);

    bot.add_adapter(CliAdapter::new(&name));
    bot.add_adapter(IrcAdapter::new(config, &name));

    let trout = handler!("TroutSlap", r"slap (?P<user>.+)", move |matches, _| {
        match matches.name("user") {
            Some(user) => {
                Some(format!("{} slaps {} around a bit with a large trout",
                             name, user))
            },
            None => None
        }
    });

    let echo = handler!("EchoHandler", r"echo (?P<msg>.+)", |matches, _| {
        matches.name("msg").map(|msg| { msg.to_owned() })
    });

    let cask_store = cask.clone();

    let info_store = handler!("InfoStore", r"(?P<key>.+) is (?P<value>.+)", move |matches, _| {
        let key = matches.name("key").unwrap();
        let value = matches.name("value").unwrap();
        cask_store.put(key.to_lowercase(), value).unwrap();
        None
    });

    let cask_recall = cask.clone();

    let info_recall = handler!("InfoRecall", r"^what is (?P<key>.+)", move |matches, _| {
        let key = matches.name("key").unwrap();
        match cask_recall.get(&key.to_lowercase()) {
            Ok(v) => match v {
                Some(v) => Some(format!("{} is {}", key, str::from_utf8(&v).unwrap())),
                None => None,
            },
            Err(_) => None,
        }
    });

    let cask_store = replies.clone();

    let reply_store = handler!("ReplyStore", r"(?P<key>.+) is <reply> (?P<value>.+)", move |matches, _| {
        let key = matches.name("key").unwrap();
        let value = matches.name("value").unwrap();
        cask_store.put(key.to_lowercase(), value).unwrap();
        None
    });

    let cask_recall = replies.clone();

    let reply_recall = handler!("ReplyRecall", r"^(?P<key>.+)", move |matches, _| {
        let key = matches.name("key").unwrap();
        match cask_recall.get(&key.to_lowercase()) {
            Ok(v) => match v {
                Some(v) => Some(str::from_utf8(&v).unwrap().to_string()),
                None => None,
            },
            Err(_) => None,
        }
    });

    bot.add_handler(ping);
    bot.add_addressed_handler(trout);
    bot.add_handler(echo);
    bot.add_handler(info_store);
    bot.add_handler(info_recall);
    bot.add_handler(reply_store);
    bot.add_handler(reply_recall);

    bot.run();
}
