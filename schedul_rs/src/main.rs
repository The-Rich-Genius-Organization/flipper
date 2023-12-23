use schedul_rs::follower::follower;
use schedul_rs::leader::leader;
use schedul_rs::redoer::redoer;
use std::env;

#[tokio::main]
async fn main() {
    let delay: u64 = match env::var("DELAY") {
        Ok(val) => val.parse().unwrap(),
        _ => 0,
    };

    let _ = match env::var("SCHED_TYPE") {
        Ok(val) if val == "leader" => leader::run().await,
        Ok(val) if val == "follower" => follower::run(delay).await,
        Ok(val) if val == "redoer" => redoer::run().await,
        _ => panic!("Invalid SCHED_TYPE env var. SCHED_TYPE must be either 'leader' or 'follower'"),
    };

    // Local Debugging
    // let leader = tokio::spawn(async {
    //     leader::run().await;
    // });
    // let follower = tokio::spawn(async {
    //     follower::run();
    // });
    // let _ = tokio::join!(leader, follower);
}
