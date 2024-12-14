use std::thread::sleep;
use std::time::Duration;

use services::BatteryManager;

fn main() {
    let mut manager = BatteryManager::new().unwrap();

    loop {
        for bat in manager.all() {
            println!(
                "Name: {:?}\nPath: {:?}\nState: {:?}\nLevel: {}",
                bat.name(),
                bat.path(),
                bat.state(),
                bat.level()
            );
            println!("{}", "-".repeat(12))
        }

        sleep(Duration::from_secs(5));
        manager.refresh().unwrap();
    }
}
