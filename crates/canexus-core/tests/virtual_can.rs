//! Black-box verification of the deterministic virtual CAN fixture interface.

use canexus_core::{
    can::{CanFormat, CanFrame, CanFrameFlags, CanId},
    event::{ChannelId, EventDirection},
    time::{ClockDomainId, TimestampOrigin, TimestampQuality, TimestampResolution},
    virtual_can::{VirtualCanAdapter, VirtualCanError, VirtualCanRecord},
    SourceId,
};

fn resolution() -> TimestampResolution {
    TimestampResolution::new(125, 2).unwrap()
}

/// A burst at one tick, a later remote request, and a sparse final frame.
/// All records are statically bounded owned frames; no external CAN device.
fn mixed_fixture() -> [VirtualCanRecord; 4] {
    let classic = CanFrame::new(
        CanId::standard(0x123).unwrap(),
        CanFrameFlags::CLASSIC,
        2,
        &[0x11, 0x22],
    )
    .unwrap();
    let fd_flags = CanFrameFlags {
        bit_rate_switch: true,
        error_state_indicator: true,
        ..CanFrameFlags::FD
    };
    let fd = CanFrame::new(
        CanId::extended(0x1abcde).unwrap(),
        fd_flags,
        9,
        &[0x55; 12],
    )
    .unwrap();
    let remote = CanFrame::new(
        CanId::standard(0x321).unwrap(),
        CanFrameFlags {
            remote: true,
            ..CanFrameFlags::CLASSIC
        },
        8,
        &[],
    )
    .unwrap();
    let empty = CanFrame::new(
        CanId::standard(0).unwrap(),
        CanFrameFlags::CLASSIC,
        0,
        &[],
    )
    .unwrap();

    [
        VirtualCanRecord {
            ticks: 10,
            channel: ChannelId::new(1),
            direction: EventDirection::Rx,
            frame: classic,
        },
        VirtualCanRecord {
            ticks: 10,
            channel: ChannelId::new(2),
            direction: EventDirection::Tx,
            frame: fd,
        },
        VirtualCanRecord {
            ticks: 1_000,
            channel: ChannelId::new(2),
            direction: EventDirection::Rx,
            frame: remote,
        },
        VirtualCanRecord {
            ticks: 1_000_000,
            channel: ChannelId::new(1),
            direction: EventDirection::Rx,
            frame: empty,
        },
    ]
}

fn adapter(records: &[VirtualCanRecord]) -> VirtualCanAdapter<'_> {
    VirtualCanAdapter::new(
        SourceId::new(7),
        ClockDomainId::new(11),
        resolution(),
        records,
    )
    .unwrap()
}

#[test]
fn replay_is_exact_and_reproducible() {
    let records = mixed_fixture();
    let first: Vec<_> = adapter(&records).collect();
    let second: Vec<_> = adapter(&records).collect();
    assert_eq!(first, second);
    assert_eq!(first.len(), 4);

    for (index, observation) in first.iter().enumerate() {
        assert_eq!(observation.id.source, SourceId::new(7));
        assert_eq!(observation.id.sequence, index as u64);
        assert_eq!(observation.channel, records[index].channel);
        assert_eq!(observation.direction, records[index].direction);
        assert_eq!(observation.timestamp.ticks, records[index].ticks);
        assert_eq!(observation.value, records[index].frame);
    }
}

#[test]
fn mixed_fixture_preserves_fd_classic_and_time_provenance() {
    let records = mixed_fixture();
    let mut replay = adapter(&records);
    let classic = replay.next().unwrap();
    assert_eq!(classic.value.data(), &[0x11, 0x22]);
    assert_eq!(classic.value.flags().format, CanFormat::Classic);

    let fd = replay.next().unwrap();
    assert_eq!(fd.channel, ChannelId::new(2));
    assert_eq!(fd.direction, EventDirection::Tx);
    assert!(fd.value.id().is_extended());
    assert_eq!(fd.value.dlc(), 9);
    assert_eq!(fd.value.data(), &[0x55; 12]);
    assert!(fd.value.flags().bit_rate_switch);
    assert!(fd.value.flags().error_state_indicator);
    assert_eq!(fd.timestamp.ticks, classic.timestamp.ticks);
    assert_eq!(fd.timestamp.domain, ClockDomainId::new(11));
    assert_eq!(fd.timestamp.resolution, resolution());
    assert_eq!(fd.timestamp.origin, TimestampOrigin::Synthetic);
    assert_eq!(fd.timestamp.quality, TimestampQuality::Uncorrelated);
    assert_eq!(fd.timestamp.delta_ticks(classic.timestamp), Some(0));

    let remote = replay.next().unwrap();
    assert!(remote.value.flags().remote);
    assert_eq!(remote.value.dlc(), 8);
    assert!(remote.value.data().is_empty());

    let last = replay.next().unwrap();
    assert_eq!(last.timestamp.ticks, 1_000_000);
    assert_eq!(last.value.data_len(), 0);
    assert_eq!(replay.next(), None);
}

#[test]
fn rejects_decreasing_ticks_but_permits_bursts() {
    let mut records = mixed_fixture();
    assert!(adapter(&records).next().is_some());
    records[2].ticks = 9;
    assert_eq!(
        VirtualCanAdapter::new(
            SourceId::new(7),
            ClockDomainId::new(11),
            resolution(),
            &records,
        )
        .unwrap_err(),
        VirtualCanError::NonMonotonicTicks {
            index: 2,
            previous: 10,
            current: 9,
        }
    );
}

#[test]
fn empty_and_exhausted_adapters_remain_exhausted() {
    let mut empty = adapter(&[]);
    assert_eq!(empty.remaining(), 0);
    assert_eq!(empty.size_hint(), (0, Some(0)));
    assert_eq!(empty.next(), None);
    assert_eq!(empty.next(), None);

    let records = mixed_fixture();
    let mut replay = adapter(&records);
    assert_eq!(replay.remaining(), 4);
    assert_eq!(replay.size_hint(), (4, Some(4)));
    for remaining in (0..4).rev() {
        assert!(replay.next().is_some());
        assert_eq!(replay.remaining(), remaining);
    }
    assert_eq!(replay.next(), None);
    assert_eq!(replay.next(), None);
}

#[test]
fn large_fixture_remains_ordered_and_supports_bursts_without_waiting() {
    let frame = CanFrame::new(
        CanId::standard(0x45).unwrap(),
        CanFrameFlags::CLASSIC,
        1,
        &[0xee],
    )
    .unwrap();
    let template = VirtualCanRecord {
        ticks: 100,
        channel: ChannelId::new(3),
        direction: EventDirection::Rx,
        frame,
    };
    let records = vec![template; 32_768];
    let replay = adapter(&records);
    assert_eq!(replay.len(), 32_768);
    let mut count = 0_u64;
    for observation in replay {
        assert_eq!(observation.id.sequence, count);
        assert_eq!(observation.timestamp.ticks, 100);
        assert_eq!(observation.value.data(), &[0xee]);
        count += 1;
    }
    assert_eq!(count, 32_768);
}

#[test]
fn independent_capture_instances_use_distinct_source_ids() {
    let records = mixed_fixture();
    let first = adapter(&records).next().unwrap();
    let second = VirtualCanAdapter::new(
        SourceId::new(8),
        ClockDomainId::new(12),
        resolution(),
        &records,
    )
    .unwrap()
    .next()
    .unwrap();
    assert_eq!(first.id.sequence, second.id.sequence);
    assert_ne!(first.id.source, second.id.source);
    assert_ne!(first.timestamp.domain, second.timestamp.domain);
}
