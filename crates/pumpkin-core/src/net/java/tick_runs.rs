//! Packets a tick phase queues for many clients, handed over once per client.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use super::{JavaClient, PacketRun};
use crate::entity::player::Player;

/// Larger packets are queued as they are, which does not copy them.
const MAX_COLLECTED_PACKET_LEN: usize = 256;

/// Numbers the sinks, so a client's place in one is not taken for its place in another.
static NEXT_PHASE: AtomicU32 = AtomicU32::new(1);

/// The number of the next sink. Never 0: a client that was in no sink yet holds 0, and
/// would be taken for the first player of a sink with that number once the counter wraps.
fn next_phase(counter: &AtomicU32) -> u32 {
    loop {
        let phase = counter.fetch_add(1, Ordering::Relaxed);
        if phase != 0 {
            return phase;
        }
    }
}

/// What one job of a phase queued, by the client's place among the players of the phase.
struct Collecting {
    phase: u32,
    runs: Vec<PacketRun>,
}

thread_local! {
    /// The collections open on this thread, innermost last. A thread that steals work
    /// starts one inside another.
    static COLLECTING: RefCell<Vec<Collecting>> = const { RefCell::new(Vec::new()) };
}

/// Collects the play packets a tick phase queues for a world's players.
///
/// Each client is queued once when the sink is dropped, so threads in a crowd do not wait
/// for each other, and still gets the packets of one job in the order they were queued.
pub struct PacketSink<'a> {
    phase: u32,
    players: &'a [Arc<Player>],
    collected: Mutex<Vec<Vec<PacketRun>>>,
}

impl<'a> PacketSink<'a> {
    #[must_use]
    pub fn new(players: &'a [Arc<Player>]) -> Self {
        let phase = next_phase(&NEXT_PHASE);
        for (place, player) in players.iter().enumerate() {
            if let Some(client) = player.client.java() {
                client.set_place(phase, place);
            }
        }
        Self {
            phase,
            players,
            collected: Mutex::default(),
        }
    }

    /// Runs `work` for every item on all threads. What it queues for `players` is handed
    /// over once it is done.
    pub fn for_each<T: Sync>(players: &[Arc<Player>], items: &[T], work: impl Fn(&T) + Sync) {
        // Few jobs, so the sink has few lists to put together.
        const JOBS: usize = 64;
        let sink = PacketSink::new(players);
        items
            .par_iter()
            .with_min_len(items.len() / JOBS + 1)
            .for_each_init(|| sink.collect(), |_, item| work(item));
    }

    /// Collects what this thread queues until the returned value is dropped.
    #[must_use]
    pub fn collect(&self) -> PacketRuns<'_> {
        COLLECTING.with_borrow_mut(|collecting| {
            collecting.push(Collecting {
                phase: self.phase,
                runs: Vec::new(),
            });
        });
        PacketRuns(self, PhantomData)
    }
}

impl Drop for PacketSink<'_> {
    fn drop(&mut self) {
        let collected = self
            .collected
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if collected.is_empty() {
            return;
        }
        let mut by_client: Vec<Vec<PacketRun>> = Vec::new();
        by_client.resize_with(self.players.len(), Vec::new);
        for job in collected.drain(..) {
            for (runs, run) in by_client.iter_mut().zip(job) {
                if !run.is_empty() {
                    runs.push(run);
                }
            }
        }
        // Each client is queued by one thread, so none of them waits for another.
        self.players
            .par_iter()
            .zip(by_client.par_iter_mut())
            .for_each(|(player, runs)| {
                if !runs.is_empty()
                    && let Some(client) = player.client.java()
                {
                    client.try_enqueue_runs(runs);
                }
            });
    }
}

/// One thread collecting into a [`PacketSink`].
pub struct PacketRuns<'a>(&'a PacketSink<'a>, PhantomData<*const ()>);

impl Drop for PacketRuns<'_> {
    fn drop(&mut self) {
        let Some(collecting) = COLLECTING.with_borrow_mut(Vec::pop) else {
            return;
        };
        if !collecting.runs.is_empty() {
            self.0
                .collected
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(collecting.runs);
        }
    }
}

/// Adds `packet` to what this thread collects for `client`. `false` when it has to be
/// queued the usual way.
pub(super) fn push(client: &JavaClient, packet: &[u8]) -> bool {
    if packet.len() > MAX_COLLECTED_PACKET_LEN {
        return false;
    }
    COLLECTING.with_borrow_mut(|collecting| {
        let Some(collecting) = collecting.last_mut() else {
            return false;
        };
        let Some(place) = client.place_in(collecting.phase) else {
            return false;
        };
        if collecting.runs.len() <= place {
            collecting.runs.resize_with(place + 1, PacketRun::default);
        }
        collecting.runs[place].push(packet);
        true
    })
}

/// Takes what this thread collected for `client` so far.
pub(super) fn take(client: &JavaClient) -> Option<PacketRun> {
    COLLECTING.with_borrow_mut(|collecting| {
        let collecting = collecting.last_mut()?;
        let run = collecting
            .runs
            .get_mut(client.place_in(collecting.phase)?)?;
        (!run.is_empty()).then(|| std::mem::take(run))
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, AtomicUsize};

    use bytes::Bytes;
    use pumpkin_data::packet::CURRENT_MC_VERSION;
    use pumpkin_protocol::RawPacket;
    use pumpkin_protocol::java::client::play::CPlayDisconnect;
    use pumpkin_protocol::java::server::play::SSwingArm;
    use pumpkin_protocol::packet::MultiVersionJavaPacket;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::text::TextComponent;

    use super::*;
    use crate::plugin::player::player_animation::PlayerAnimationEvent;
    use crate::plugin::server::packet::PacketSentEvent;
    use crate::plugin::{BoxFuture, EventHandler, EventPriority};
    use crate::server::Server;
    use crate::test_support::{Client, TestResult, server, shutdown};
    use crate::world::World;

    /// A packet with id `id` and `payload` as its only field.
    fn numbered(id: u8, payload: u32) -> Bytes {
        let mut data = vec![id];
        data.extend_from_slice(&payload.to_be_bytes());
        data.into()
    }

    fn java(client: &Client) -> &JavaClient {
        client.player.client.java().expect("a Java player")
    }

    async fn ids(client: &mut Client) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
        Ok(client.received().await?.iter().map(|p| p.id).collect())
    }

    #[test]
    fn no_sink_gets_the_number_of_a_client_in_no_sink() {
        let counter = AtomicU32::new(u32::MAX);
        assert_eq!(next_phase(&counter), u32::MAX);
        assert_eq!(next_phase(&counter), 1);
    }

    #[tokio::test]
    async fn a_job_queues_each_client_once_and_in_its_own_order() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let mut first = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
        let mut second = Client::join(&server, 2, Vector3::new(40.5, 100.0, 0.5));
        first.received().await?;
        second.received().await?;
        java(&first).suspend_flushing();

        let players = server.worlds.load()[0].players.load_full();
        let sink = PacketSink::new(&players);
        let runs = sink.collect();
        java(&first).try_enqueue_packet(numbered(1, 0));
        java(&second).try_enqueue_packet(numbered(2, 0));
        java(&first).try_enqueue_packet(numbered(3, 0));
        // Not collected: one waits for the writer, one is too large to copy.
        assert!(java(&first).send_packet_now_data(numbered(4, 0)).await);
        java(&first).try_enqueue_packet(numbered(5, 0));
        let mut large = vec![6];
        large.resize(MAX_COLLECTED_PACKET_LEN + 1, 0);
        java(&first).try_enqueue_packet(large.into());
        java(&first).try_enqueue_packet(numbered(7, 0));
        drop(runs);
        drop(sink);
        java(&first).resume_flushing();

        assert_eq!(ids(&mut first).await?, [1, 3, 4, 5, 6, 7]);
        assert_eq!(ids(&mut second).await?, [2]);
        // Each collected packet counted once: nothing stays reserved once all is written.
        java(&first).send_packet_now_data(numbered(8, 0)).await;
        assert_eq!(java(&first).pending_bytes.load(Ordering::Acquire), 0);
        shutdown(&server).await;
        Ok(())
    }

    #[tokio::test]
    async fn a_kick_follows_what_was_collected_before_it() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let mut client = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
        client.received().await?;
        java(&client).suspend_flushing();

        let players = server.worlds.load()[0].players.load_full();
        let sink = PacketSink::new(&players);
        let runs = sink.collect();
        java(&client).try_enqueue_packet(numbered(1, 0));
        java(&client).try_kick(&TextComponent::text("bye"));
        drop(runs);
        drop(sink);

        // A kicked client never gets the end marker of `received`.
        assert_eq!(client.next_packet().await?.id, 1);
        assert_eq!(
            client.next_packet().await?.id,
            CPlayDisconnect::to_id(CURRENT_MC_VERSION)
        );
        shutdown(&server).await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn threads_collecting_for_one_client_keep_their_own_order() -> TestResult {
        const THREADS: u8 = 4;
        const PACKETS: u32 = 500;
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let mut client = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
        client.received().await?;
        java(&client).suspend_flushing();

        let players = server.worlds.load()[0].players.load_full();
        let sink = PacketSink::new(&players);
        std::thread::scope(|scope| {
            for thread in 1..=THREADS {
                let (player, sink) = (&client.player, &sink);
                scope.spawn(move || {
                    let Some(java) = player.client.java() else {
                        return;
                    };
                    // Several jobs a thread, so those of different threads interleave.
                    for part in 0..PACKETS / 100 {
                        let _runs = sink.collect();
                        for packet in part * 100..(part + 1) * 100 {
                            java.try_enqueue_packet(numbered(thread, packet));
                        }
                    }
                });
            }
        });
        drop(sink);
        java(&client).resume_flushing();

        let mut next = [0u32; THREADS as usize + 1];
        for packet in client.received().await? {
            let payload: [u8; 4] = packet.payload[..4].try_into()?;
            let thread = packet.id as usize;
            assert_eq!(u32::from_be_bytes(payload), next[thread]);
            next[thread] += 1;
        }
        assert_eq!(next[1..], [PACKETS; THREADS as usize]);
        shutdown(&server).await;
        Ok(())
    }

    struct CountsPackets(AtomicU32);

    impl EventHandler<PacketSentEvent> for CountsPackets {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            _event: &'a PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                self.0.fetch_add(1, Ordering::Relaxed);
            })
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_plugin_watching_packets_still_sees_each_one() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let mut client = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
        client.received().await?;
        let seen = Arc::new(CountsPackets(AtomicU32::new(0)));
        server
            .plugin_manager
            .register(seen.clone(), EventPriority::Normal, false);

        let players = server.worlds.load()[0].players.load_full();
        let sink = PacketSink::new(&players);
        let runs = sink.collect();
        for id in 1..=3 {
            java(&client).try_enqueue_packet(numbered(id, 0));
        }
        drop(runs);
        drop(sink);

        assert_eq!(ids(&mut client).await?, [1, 2, 3]);
        assert_eq!(seen.0.load(Ordering::Relaxed), 3);
        shutdown(&server).await;
        Ok(())
    }

    /// Stands in for a plugin that tells another player about an arm swing.
    struct TellsWatcher {
        watcher: Arc<Player>,
        /// The watcher's pending bytes right after the handler queued its packet.
        pending: AtomicUsize,
    }

    impl EventHandler<PlayerAnimationEvent> for TellsWatcher {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerAnimationEvent,
        ) -> BoxFuture<'a, ()> {
            // Cancelled, so the swing itself queues nothing.
            event.cancelled = true;
            if let Some(java) = self.watcher.client.java() {
                java.try_enqueue_packet(numbered(1, 0));
                let pending = java.pending_bytes.load(Ordering::Acquire);
                self.pending.store(pending, Ordering::Relaxed);
            }
            Box::pin(async {})
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_packet_handler_queues_for_another_player_at_once() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let sender = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
        let watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
        // Written, so nothing of the join is pending any more. Mid-tick nothing is
        // written, so what a handler queues stays pending.
        assert!(java(&watcher).send_packet_now_data(numbered(2, 0)).await);
        java(&watcher).suspend_flushing();
        let plugin = Arc::new(TellsWatcher {
            watcher: watcher.player.clone(),
            pending: AtomicUsize::new(0),
        });
        server
            .plugin_manager
            .register(plugin.clone(), EventPriority::Normal, true);

        sender.player.inbound_packets.push(RawPacket {
            id: SSwingArm::to_id(CURRENT_MC_VERSION),
            payload: Bytes::new(),
        });
        let players = server.worlds.load()[0].players.load_full();
        World::tick_players(&players, &server);

        // Queued when the handler ran, not once the packets of all players were handled:
        // then it would follow what happened after it.
        assert_ne!(plugin.pending.load(Ordering::Relaxed), 0);
        shutdown(&server).await;
        Ok(())
    }
}
