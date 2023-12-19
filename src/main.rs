use twitchchat::Config;
use twitchchat::message::MessageType;
use twitchchat::stream::ChatStream;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    let channel = std::env::var("CHANNEL").expect("No CHANNEL env set");
    let token = std::env::var("TOKEN").expect("No TOKEN env set");
    let name = std::env::var("NAME").expect("No NAME env set");
    let perms = vec!["commands".to_string(), "tags".to_string()];
    let channels = vec![channel.to_string()];

    let cfg = Config::new(
        token.to_string(), 
        name.to_string(), 
        channels,
        perms);

    let stream = ChatStream::connect(&cfg).unwrap();
    let handle = stream.on_message(|msg| {
        println!("\n\
            User - {},\n\
            MessageType - {:?},\n\
            Message - {},\n\
            Tags - {:#?},\n\
            Badges - {:#?}\n", 
            msg.user, msg.kind, msg.message, msg.tags.tags, msg.tags.badges);
    });

    if let Ok(handle) = handle {
        let _ = handle.join();
    };
}
