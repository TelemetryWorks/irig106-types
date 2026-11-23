//! PF-001/002: API layout budget, not the packed wire layout in Table 11-2.
//! Compile this test on wasm32 too: the const assertion evaluates on the
//! target even when a wasm test runner is unavailable. Native tests execute.
use core::mem::size_of;
use irig106_types::PacketFacts;

const _: () = assert!(size_of::<PacketFacts>() <= 32);

#[test]
fn packet_facts_fits_the_reviewed_budget() {
    assert!(size_of::<PacketFacts>() <= 32);
    println!(
        "PacketFacts: {} bytes on {}",
        size_of::<PacketFacts>(),
        std::env::consts::ARCH
    );
}
