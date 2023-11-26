use schedul_rs::follower;
use schedul_rs::leader;

#[tokio::main]
async fn main() {
    let leader = tokio::spawn(async {
        leader::run().await;
    });
    let follower = tokio::spawn(async {
        follower::run();
    });

    let _ = tokio::join!(leader, follower);
}
