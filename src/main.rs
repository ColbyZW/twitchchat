use twitchchat::Config;
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
        perms
    );

    let stream = ChatStream::connect(&cfg).unwrap();
    let _ = stream.on_message(|msg| {
        println!("{}: {}", msg.user, msg.message);
    });

    stream.listen();
}
