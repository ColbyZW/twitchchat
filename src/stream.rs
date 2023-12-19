use std::net::{TcpStream};
use std::sync::{Mutex, Arc};
use std::io::{Write, BufRead, BufReader};
use std::ops::DerefMut;
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;
use crate::{Config};
use crate::message::{ChatMessage, MessageType};

pub struct ChatStream {
    pub stream: Arc<Mutex<TcpStream>>
}

impl ChatStream {
    // Opens the initial TCP connection to the twitch IRC
    pub fn connect(cfg: &Config) -> Result<ChatStream, &'static str> {
        if let Ok(mut stream) = TcpStream::connect("irc.chat.twitch.tv:6667") {
            if let Err(_) = ChatStream::handshake(&cfg, &mut stream) {
                return Err("Unable to perform handshake");
            };

            if let Err(_) = stream.set_read_timeout(Some(Duration::from_secs(60 * 10))) {
                return Err("Unable to set timeout on TCP Stream");
            };

            let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(stream));

            return Ok(ChatStream { stream });
        } else {
            return Err("Unable to connect to specified URL");
        };
    }

    pub fn ping(msg: &ChatMessage) {
        let pong_message = String::from("PONG ") + &msg.message + "\r\n";
        println!("Sending PONG");
        let mut stream = msg.stream.lock().unwrap();
        let _ = stream.write(pong_message.as_bytes());
        if let Err(_) = stream.flush() {
            panic!("Failed to reply to PING");
        };
    }

    fn handle_message(
        message: &ChatMessage, 
        handler: fn(&ChatMessage) -> ()
        ) {
         match message.kind {
            MessageType::PING => {
                ChatStream::ping(&message);
            },
            _ => {
                handler(&message);
            },
        };
    }

    // Registers a callback to run on messages
    pub fn on_message(
        self: &Self, 
        handler: fn(&ChatMessage) -> ()
        ) -> Result<JoinHandle<()>, &'static str> {
        let stream = self.stream.clone();

        if let Ok(write_stream) = self.stream.lock().unwrap().try_clone() {
            let handle = thread::spawn(move || {
                let mut stream = stream.lock().unwrap();
                let reader = BufReader::new(stream.deref_mut());
                let write_stream = Arc::new(Mutex::new(write_stream));

                for res in reader.lines() {
                    if let Ok(line) = res {
                        let msg = ChatMessage::new(&line, &write_stream);
                        ChatStream::handle_message(&msg, handler);
                    };
                    thread::sleep(Duration::from_millis(1));
                }
            });

            Ok(handle)
        } else {
            Err("Unable to get Mutable clone of Stream")
        }
    }

    pub fn send(self: &Self, message: &str, channel: &str) -> Result<(), &'static str> {
        let mut stream = self.stream.lock().unwrap();
        let msg = format!("PRIVMSG #{} :{}\r\n", channel, message);
        let _ = stream.write(msg.as_bytes());
        if let Err(_) = stream.flush() {
            return Err("Unable to send message");
        } else {
            return Ok(());
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
