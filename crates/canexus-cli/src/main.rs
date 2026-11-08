//! Minimal headless entry point used to validate workspace wiring.

use canexus_core::SourceId;

fn main() {
    let source = SourceId::new(0);
    println!("CANexus-Studio bootstrap (source {})", source.get());
}
