pub mod message;
pub mod stream;

pub struct Config {
    pub pass: String,
    pub nick: String,
    pub channels: Vec<String>,
    pub perms: Vec<String>
}

impl Config {
    pub fn new(
        pass: String,
        nick: String,
        channels: Vec<String>,
        perms: Vec<String>
        ) -> Self {
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


