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
    loop {
        let start = Instant::now();

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
            // while we're sending to the database, also update the outbound payload

            if outpayload.send(Arc::new(pl)).is_err() {
                // happens if the subscribers disconnect while we prepared the payload
                warn!("Lost all world subscribers");
            }
        }

        info!("Tick done in {:?}", Instant::now() - start);

        let sleep_duration = tick_latency
            .checked_sub(Instant::now() - start)
            .unwrap_or_else(|| Duration::from_millis(0));

        debug!("Sleeping for {:?}", sleep_duration);
        tokio::time::sleep(sleep_duration).await;
    }
}
