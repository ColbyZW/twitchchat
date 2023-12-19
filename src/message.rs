use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::net::{TcpStream};
use std::io::{Write};

#[derive(Debug)]
pub enum MessageType {
    PING,
    PRIVMSG,
    NONE
}

pub struct ChatMessage {
    pub user: String,
    pub room: String,
    pub message: String,
    pub kind: MessageType,
    pub tags: Tags,
    pub stream: Arc<Mutex<TcpStream>>,
}

impl ChatMessage {
    pub fn reply(self: &Self, msg: &str) -> Result<(), &'static str> {
        match self.kind {
            MessageType::PRIVMSG =>    {
                let mut stream = self.stream.lock().unwrap();
                let msg_id = self.tags.tags.get("id").unwrap();
                let reply = format!("@reply-parent-msg-id={} PRIVMSG {} :{}\r\n",
                    msg_id, self.room, msg);

                let _ = stream.write(reply.as_bytes());

                if let Err(_) = stream.flush() {
                    return Err("Unable to reply to message");
                };

                return Ok(());
            },
            _ => {
                return Ok(());
            }
        };
    }

    pub fn new(message: &str, stream: &Arc<Mutex<TcpStream>>) -> Self {
        let mut idx = 0;
        let mut chat_msg = ChatMessage {
            user: String::new(),
            room: String::new(),
            message: String::new(),
            kind: MessageType::NONE,
            tags: Tags::empty(),
            stream: Arc::clone(stream)
        };

        if &message[0..1] == "@" {
            if let Some(end) = message.find(" ") {
                let tags_string = &message[idx+1..end];
                idx = end+1;
                let tags: Tags = Tags::new(&tags_string);
                chat_msg.tags = tags;
            }
        }


        let message = &message[idx..];
        if let Some(colon) = message.find(":") {
            if let Some(end) = message.find(" ") {
                if end > colon {
                    let source = &message[colon+1..end];
                    idx = end;
                    let user: Vec<&str> = source.split("!").collect();
                    if let Some(name) = user.get(0) {
                        chat_msg.user.push_str(name);
                    };
                }
            };
        };


        let message = &message[idx..];
        if let Some(end) = message.find(":") {
            let command = &message[..end].trim();
            let cmd: Vec<&str> = command.split(" ").collect();
            if let Some(kind) = cmd.get(0) {
                if *kind == "PRIVMSG" {
                    chat_msg.kind = MessageType::PRIVMSG;
                    if let Some(room) = cmd.get(1) {
                        chat_msg.room.push_str(room);
                    };
                } else if *kind == "PING" {
                    chat_msg.kind = MessageType::PING;
                }
            };

            let parameters = &message[end+1..];
            chat_msg.message.push_str(parameters);
        };

        chat_msg
    }
}


pub struct Tags {
    pub badges: HashMap<String, HashMap<String, String>>,
    pub tags: HashMap<String, String>
}

impl Tags {
    pub fn new(tags: &str) -> Self {
        let mut tag_set: HashMap<String, String> = HashMap::new();
        let mut badge_set: HashMap<String, HashMap<String, String>> = HashMap::new();
        let parsed_tags: Vec<&str> = tags.split(";").collect();

        for tag in parsed_tags {
            let tag_pair: Vec<&str> = tag.split("=").collect();
            if let Some(value) = tag_pair.get(1) {

                if let Some(key) = tag_pair.get(0) {

                    if *key == "badges" || *key == "badge-info" {
                        let badges: Vec<&str> = value.split(",").collect();

                        for badge in badges {
                            let mut sub_badge_set: HashMap<String, String> = HashMap::new();
                            let badge_parts: Vec<&str> = badge.split('/').collect();
                            if badge_parts.len() >= 2 {

                                sub_badge_set.insert(badge_parts[0].to_string(), badge_parts[1].to_string());
                            }
                            badge_set.insert((*key).to_string(), sub_badge_set);
                        }
                    } else {
                        tag_set.insert((*key).to_string(), value.to_string());
                    }
                };
            };
        }

        Self {
            tags: tag_set,
            badges: badge_set
        }
    }

    pub fn empty() -> Self {
        Self {
            tags: HashMap::new(),
            badges: HashMap::new()
        }
    }
}

