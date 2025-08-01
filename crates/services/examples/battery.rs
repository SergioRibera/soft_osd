use std::thread::sleep;
use std::time::Duration;

use services::BatteryManager;

#[tokio::main]
async fn main() {
    // this support bluetooth too
    let manager = BatteryManager::new(|changed| {
        println!(
            "Battery charger: {:?}",
            if changed { "Connected" } else { "Disconnected" }
        );
    })
    .await
    .unwrap();

    loop {
        for bat in manager.all() {
            println!(
                "Name: {:?}\nPath: {:?}\nState: {:?}\nLevel: {}",
                bat.name(),
                bat.path(),
                bat.state(),
                bat.level()
            );
            println!("{}", "-".repeat(12));
        }
        println!("{}", "#".repeat(20));

        sleep(Duration::from_secs(5));
        manager.refresh().await.unwrap();
    }
}
