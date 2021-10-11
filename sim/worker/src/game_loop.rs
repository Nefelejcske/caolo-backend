use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use caolo_sim::executor::SimpleExecutor;
use tokio::sync::broadcast::Sender;
use tracing::{debug, info, warn};

use crate::{world_service, WorldContainer};

pub async fn game_loop(
    world: WorldContainer,
    mut executor: SimpleExecutor,
    outpayload: Arc<Sender<Arc<world_service::Payload>>>,
    tick_latency: Duration,
) {
    let mut lag = Duration::new(0, 0);
    loop {
        let start = Instant::now();

        #[cfg(save_world)]
        {
            // save the latest world state on a background thread
            // TODO use two files and double-buffer based on time()?
            // so if save fails we'll still have the one-before the last save
            let world = world.clone();
            tokio::spawn(async move {
                let start = Instant::now();
                let world_guard = world.read().await;
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open("latest_world.bin")
                    .unwrap();
                bincode::serialize_into(&mut f, &*world_guard).unwrap();
                drop(world_guard);

                let end = Instant::now();

                info!("Saved current world state in {:?}", end - start);
            });
        }

        let world_guard = world.read().await;
        let sp = tracing::error_span!("game-loop", tick = world_guard.time());
        let _e = sp.enter();

        let intents = executor.forward_bots(&world_guard).await.unwrap();
        drop(world_guard); // free the read guard

        // NOTE: commands may be executed between `forward_bots` and `apply_intents`
        // allow this for now, but may be worth revisiting

        let mut world_guard = world.write().await;
        executor
            .apply_intents(&mut world_guard, intents)
            .await
            .unwrap();
        drop(world_guard); // free the write guard

        let world_guard = world.read().await;
        let mut pl = world_service::Payload::default();
        pl.update(&world_guard);
        drop(world_guard); // free the read guard

        if outpayload.receiver_count() > 0 {
            debug!("Sending world entities to subscribers");
            if outpayload.send(Arc::new(pl)).is_err() {
                // happens if the subscribers disconnect while we sent the payload
                warn!("Lost all world subscribers");
            }
        }

        let end = Instant::now();
        let tick_duration = end - start;

        let mut sleep_duration = tick_latency.checked_sub(tick_duration).unwrap_or_default();
        if tick_duration < tick_latency {
            lag = lag
                .checked_sub(tick_latency - tick_duration)
                .unwrap_or_default();
            if !lag.is_zero() {
                sleep_duration = Duration::from_millis(0);
            }
        } else {
            lag += tick_duration - tick_latency;
            sleep_duration = Duration::from_millis(0);
        }
        info!(
            "Tick done in {:.2?}. Current lag: {:.2?}",
            tick_duration, lag
        );

        tokio::time::sleep(sleep_duration).await;
    }
}
