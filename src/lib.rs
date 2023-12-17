use std::net::{TcpStream};
use std::io::{Write, BufRead, BufReader};
use std::thread;
use std::ops::DerefMut;
use std::sync::{Mutex, Arc};
use std::thread::JoinHandle;
use std::time::Duration;
use std::collections::HashMap;

pub struct Config {
    pub pass: String,
    pub nick: String,
    pub channels: Vec<String>,
    pub perms: Vec<String>
}

impl Config {
    pub fn new(pass: String, nick: String, channels: Vec<String>, perms: Vec<String>) -> Self {
        Self {pass, nick, channels, perms}
    }

    pub fn create_perm_string(self: &Self) -> String {
        let mut perm_string = String::from("CAP REQ :");
        for perm in &self.perms {
            perm_string.push_str(&(String::from("twitch.tv/") + &perm + " "));
        }
        return perm_string + "\r\n";
    }

    pub fn create_pass_string(self: &Self) -> String {
        let pass_string = String::from("PASS oauth:") + &self.pass;
        return pass_string + "\r\n";
    }

    pub fn create_nick_string(self: &Self) -> String {
        let nick_string = String::from("NICK ") + &self.nick;
        return nick_string + "\r\n";
    }

    pub fn create_join_string(self: &Self) -> String {
        let mut join_string = String::from("JOIN ");
        for channel in &self.channels {
            join_string.push_str(&(String::from("#") + &channel + ","));
        }
        return join_string + "\r\n";
    }
}

#[derive(Debug)]
pub enum MessageType {
    PING,
    PRIVMSG,
    NONE
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

pub struct ChatMessage {
    pub user: String,
    pub message: String,
    pub kind: MessageType,
    pub tags: Tags
}

impl ChatMessage {
    pub fn new(message: &str) -> Self {
        let mut idx = 0;
        let mut chat_msg = ChatMessage {
            user: String::new(),
            message: String::new(),
            kind: MessageType::NONE,
            tags: Tags::empty()
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
                let source = &message[colon+1..end];
                idx = end;
                let user: Vec<&str> = source.split("!").collect();
                if let Some(name) = user.get(0) {
                    chat_msg.user.push_str(name);
                };
            };
        };


        let message = &message[idx..];
        if let Some(end) = message.find(":") {
            let command = &message[..end].trim();
            let cmd: Vec<&str> = command.split(" ").collect();
            if let Some(kind) = cmd.get(0) {
                if *kind == "PRIVMSG" {
                    chat_msg.kind = MessageType::PRIVMSG;
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


pub struct ChatStream {
    pub stream: Arc<Mutex<TcpStream>>
}

impl ChatStream {
    // Opens the initial TCP connection to the twitch IRC
    pub fn connect(cfg: &Config) -> Result<ChatStream, &'static str> {
        if let Ok(mut stream) = TcpStream::connect("irc.chat.twitch.tv:6667") {
            if let Ok(()) = ChatStream::handshake(&cfg, &mut stream) {
                if let Ok(_) = stream.set_read_timeout(Some(Duration::from_secs(60 * 10))) {
                    let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(stream));
                    return Ok(ChatStream { stream });
                } else {
                    return Err("Unable to set timeout on TcpStream");
                };
            } else {
                return Err("Unable to authenticate with server");
            }
        } else {
            return Err("Unable to connect to specified URL");
        };
    }

    pub fn ping(stream: &mut TcpStream, msg: &ChatMessage) {
        let pong_message = String::from("PONG ") + &msg.message + "\r\n";
        println!("Sending PONG");
        let _ = stream.write(pong_message.as_bytes());
        if let Err(_) = stream.flush() {
            panic!("Failed to reply to PING");
        };
    }

    fn handle_message(
        stream: &mut TcpStream,
        message: &ChatMessage, 
        handler: fn(&ChatMessage) -> ()
        ) {
         match message.kind {
            MessageType::PING => {
                ChatStream::ping(stream, &message);
            },
            MessageType::PRIVMSG => {
                handler(&message);
            },
            MessageType::NONE => {
                handler(&message);
            },
        };
    }

    // Registers a callback to run on messages
    pub fn on_message(self: &Self, handler: fn(&ChatMessage) -> ()) 
        -> Result<JoinHandle<()>, &'static str> {
        let stream = self.stream.clone();
        if let Ok(mut write_stream) = self.stream.lock().unwrap().try_clone() {
            let handle = thread::spawn(move || {
                let mut stream = stream.lock().unwrap();
                let reader = BufReader::new(stream.deref_mut());
                for res in reader.lines() {
                    if let Ok(line) = res {
                        let msg = ChatMessage::new(&line);
                        ChatStream::handle_message(&mut write_stream, &msg, handler);
                    };
                }
            });
            return Ok(handle);
        } else {
            return Err("Unable to get Mutable clone of Stream");
        }
    }

    // Performs the twitch IRC handshake
    fn handshake(cfg: &Config, stream: &mut TcpStream) -> Result<(), &'static str> {
        let pass_msg = cfg.create_pass_string();
        let nick_msg = cfg.create_nick_string();
        let perm_msg = cfg.create_perm_string();
        let join_msg = cfg.create_join_string();

        let _ = stream.write(perm_msg.as_bytes());
        let _ = stream.write(pass_msg.as_bytes());
        let _ = stream.write(nick_msg.as_bytes());
        let _ = stream.write(join_msg.as_bytes());
        if let Err(_) = stream.flush() {
            println!("Failed to send messages to Twitch");
        } else {
            println!("Wrote all messages to twitch!");
        }

        Ok(())
    }
}
