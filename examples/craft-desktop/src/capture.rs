use canexus_core::{
    can::CanFrame,
    event::Observed,
    ingest::{BatchBudget, IngestConfig, IngestError},
    session::{CaptureSession, FlushClock, SessionConfig, SessionError, Subscription},
    time::{ClockDomainId, TimestampResolution},
    trace_history::{HistoryLimits, PageRequest, TraceHistory},
    virtual_can::{fixtures, VirtualCanAdapter},
    SourceId,
};
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

const SOURCE: SourceId = SourceId::new(1);
pub const ROW_LIMIT: usize = 512;
const PERIOD: Duration = Duration::from_millis(75);

#[derive(Clone, Default)]
pub struct Snapshot {
    pub rows: Arc<[Observed<CanFrame>]>,
    pub accepted: u64,
    pub rejected: u64,
    pub receiver_loss: u64,
    pub evicted: u64,
    pub error: Option<String>,
}

pub struct Capture {
    stop: Option<mpsc::SyncSender<()>>,
    thread: JoinHandle<()>,
}

impl Capture {
    pub fn start(output: Arc<Mutex<Snapshot>>) -> io::Result<Self> {
        let (stop, command) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("virtual-can".into())
            .spawn(move || {
                if let Err(error) = run(&command, &output) {
                    output.lock().expect("trace snapshot").error = Some(error);
                }
            })?;
        Ok(Self {
            stop: Some(stop),
            thread,
        })
    }

    pub fn stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.try_send(());
        }
    }

    pub fn is_stopping(&self) -> bool {
        self.stop.is_none()
    }

    pub fn is_finished(&self) -> bool {
        self.thread.is_finished()
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(command: &mpsc::Receiver<()>, output: &Mutex<Snapshot>) -> Result<(), String> {
    let config = SessionConfig {
        ingest: IngestConfig::new(4, 32, 1).expect("fixed ingest limits"),
        flush_after: Duration::from_millis(15),
        stop_grace: Duration::from_millis(300),
        clock: FlushClock::RealTime,
    };
    let session = CaptureSession::start(config, BatchBudget::new(2 * 1024 * 1024), &[SOURCE])
        .map_err(|error| format!("Cannot start capture: {error:?}"))?;
    let receiver = session.subscribe(SOURCE, 32).expect("fixed subscriber");
    let limits = HistoryLimits::new(
        ROW_LIMIT * std::mem::size_of::<Observed<CanFrame>>(),
        ROW_LIMIT,
        ROW_LIMIT,
    )
    .expect("fixed trace limits");
    let mut history = TraceHistory::new(SOURCE, limits);
    let fixtures = fixtures::mixed_classic_fd();
    let input = session.input();
    let mut sequence = 0_u64;
    loop {
        let mut fixture = fixtures[(sequence % fixtures.len() as u64) as usize];
        // Millisecond ticks in the synthetic domain are independent of OS scheduling.
        fixture.at_tick = sequence
            .checked_mul(PERIOD.as_millis() as u64)
            .ok_or("Virtual clock exhausted")?;
        let mut adapter = VirtualCanAdapter::new(
            SOURCE,
            ClockDomainId::new(1),
            TimestampResolution::new(1_000_000, 1).expect("millisecond resolution"),
            std::slice::from_ref(&fixture),
        )
        .expect("single fixture");
        adapter.start().expect("new adapter");
        adapter.advance_to(fixture.at_tick).expect("monotonic tick");
        let mut event = adapter
            .next_due()
            .expect("running adapter")
            .expect("due event");
        event.id.sequence = sequence;
        match input.submit(input.generation(), event) {
            Ok(()) | Err(SessionError::Ingest(IngestError::BatchBudgetExhausted)) => {}
            Err(error) => return Err(format!("Capture input failed: {error:?}")),
        }
        sequence = sequence
            .checked_add(1)
            .ok_or("Virtual sequence exhausted")?;
        match command.recv_timeout(PERIOD) {
            Err(mpsc::RecvTimeoutError::Timeout) => {
                while let Ok(delivery) = receiver.recv_timeout(Duration::ZERO) {
                    append(&mut history, delivery.batch)?;
                }
                publish(output, &history, &session, &receiver)?;
            }
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    thread::scope(|scope| {
        scope.spawn(|| session.stop());
        while let Ok(delivery) = receiver.recv_timeout(config.stop_grace) {
            append(&mut history, delivery.batch)?;
        }
        Ok::<(), String>(())
    })?;
    publish(output, &history, &session, &receiver)
}

fn append(
    history: &mut TraceHistory<CanFrame>,
    batch: canexus_core::ingest::SharedBatch<CanFrame>,
) -> Result<(), String> {
    history
        .append(batch)
        .map_err(|error| format!("Trace history: {error:?}"))?;
    Ok(())
}

fn publish(
    output: &Mutex<Snapshot>,
    history: &TraceHistory<CanFrame>,
    session: &CaptureSession,
    receiver: &Subscription,
) -> Result<(), String> {
    let status = session.status();
    let rows = history
        .page_with(
            PageRequest {
                cursor: None,
                limit: ROW_LIMIT,
            },
            |_| true,
        )
        .map_err(|error| format!("Trace page: {error:?}"))?
        .rows
        .into_iter()
        .copied()
        .collect();
    *output.lock().expect("trace snapshot") = Snapshot {
        rows,
        accepted: status.sources[0].ingest.accepted_events,
        rejected: status.sources[0].ingest.rejected_events,
        receiver_loss: receiver
            .status()
            .expect("attached receiver")
            .delivery
            .dropped_events,
        evicted: history.stats().evicted_events,
        error: None,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use canexus_core::can::CanFormat;
    use std::time::Instant;

    #[test]
    fn running_worker_publishes_classic_and_fd_then_drains() {
        let snapshot = Arc::new(Mutex::new(Snapshot::default()));
        let mut capture = Capture::start(Arc::clone(&snapshot)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let current = snapshot.lock().unwrap().clone();
            assert!(current.error.is_none(), "{:?}", current.error);
            if current
                .rows
                .iter()
                .any(|event| event.value.flags().format == CanFormat::Fd)
            {
                break;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        capture.stop();
        while !capture.is_finished() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        let result = snapshot.lock().unwrap();
        assert!(result.error.is_none());
        assert!(result
            .rows
            .iter()
            .any(|event| event.value.flags().format == CanFormat::Classic));
        assert_eq!(result.accepted, result.rows.len() as u64);
        assert_eq!(result.receiver_loss, 0);
        assert!(result
            .rows
            .windows(2)
            .all(|pair| pair[0].id.sequence + 1 == pair[1].id.sequence));
    }
}
