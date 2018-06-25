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


    let name = "robotron";
    let mut bot = Chatbot::new(name);

    bot.add_adapter(CliAdapter::new(name));
    {
        let config = chatbot::adapter::IrcConfig {
            nickname: Some(format!("{}", name)),
            alt_nicks: Some(vec![format!("{}_", name), format!("{}__", name)]),
            server: Some(format!("chat.freenode.net")),
            channels: Some(vec![format!("#whatme")]),
            .. Default::default()
        };
        bot.add_adapter(IrcAdapter::new(config, name))
    };

    let ping = handler!("PingHandler", r"ping", |_, _| { Some("pong".to_owned()) });

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
        match cask_recall.get(&key) {
            Ok(v) => Some(str::from_utf8(&v.unwrap()).unwrap().to_string()),
            Err(_) => None,
        }
    });

    bot.add_handler(ping);
    bot.add_addressed_handler(trout);
    bot.add_handler(echo);
    bot.add_handler(info_store);
    bot.add_handler(info_recall);

    bot.run();
}
