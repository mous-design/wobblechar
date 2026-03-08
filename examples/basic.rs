//! Basic usage examples for `wobblechar`.

use wobblechar::{Builder, Entry};

fn main() {
    bool_mapper();
    num_mapper();
    multi_line_labeled();
    custom_const_map();
}

/// Single-line waveform decoded as `bool` values.
fn bool_mapper() {
    println!("--- bool mapper ---");
    for item in Builder::<1>::new_from_string("_|‾|_")
        .with_def_bool_mapper()
        .build()
    {
        println!("t={} value={} changed={}", item.index, item.values[0], item.changed);
    }
    // t=0 value=false changed=false
    // t=1 value=true  changed=true
    // t=2 value=true  changed=false
    // t=3 value=false changed=true
    // t=4 value=false changed=false
}

/// Two-line waveform decoded as `u8` values (0 / 1).
fn num_mapper() {
    println!("--- num mapper (u8) ---");
    for item in Builder::<2>::new_from_string("
        _|‾|_
        ‾|_|‾
    ")
        .with_def_num_mapper::<u8>()
        .build()
    {
        println!("t={} values={:?} changed={}", item.index, item.values, item.changed);
    }
    // t=0 values=[0, 1] changed=false
    // t=1 values=[1, 0] changed=true
    // t=2 values=[1, 0] changed=false
    // t=3 values=[0, 1] changed=true
    // t=4 values=[0, 1] changed=false
}

/// Multi-line labeled format: the same label can appear on separate blocks,
/// and `wobblechar` stitches them together transparently.
fn multi_line_labeled() {
    println!("--- labeled multi-line ---");
    for item in Builder::<2>::new_from_string("
        CLK: _|‾|_|‾|_   # first block
        DAT: ___|‾‾‾|_

        CLK: ‾|_|‾|_|‾   # continuation
        DAT: |‾‾‾|__|‾
    ")
        .with_def_bool_mapper()
        .build()
    {
        println!(
            "t={:02} CLK={} DAT={} changed={}",
            item.index, item.values[0], item.values[1], item.changed
        );
    }
    // t=00 CLK=false DAT=false changed=false
    // t=01 CLK=true  DAT=false changed=true
    // t=02 CLK=true  DAT=false changed=false
    // t=03 CLK=false DAT=true changed=true
    // t=04 CLK=false DAT=true changed=false
    // t=05 CLK=true  DAT=true changed=true
    // t=06 CLK=true  DAT=true changed=false
    // t=07 CLK=false DAT=false changed=true
    // t=08 CLK=false DAT=false changed=false
    // t=09 CLK=true  DAT=true changed=true
    // t=10 CLK=false DAT=true changed=true
    // t=11 CLK=false DAT=true changed=false
    // t=12 CLK=true  DAT=true changed=true
    // t=13 CLK=true  DAT=false changed=true
    // t=14 CLK=false DAT=false changed=true
    // t=15 CLK=false DAT=false changed=false
    // t=16 CLK=true  DAT=true changed=true
    // t=17 CLK=true  DAT=true changed=false
}

/// Custom character map using a `const` slice — works in `no_std` environments.
///
/// Maps: `H` → true (high), `L` → false (low), `|` → toggle (edge).
fn custom_const_map() {
    println!("--- custom const map ---");
    const MAP: &[(char, Entry<bool>)] = &[
        ('H', Entry::Value(true)),
        ('L', Entry::Value(false)),
        ('|', Entry::Toggle),
    ];
    for item in Builder::<1>::new_from_string("LHL|H|L")
        .with_const_bool_map(MAP)
        .build()
    {
        println!("t={} value={} changed={}", item.index, item.values[0], item.changed);
    }
    // t=0 value=false changed=false
    // t=1 value=true changed=true
    // t=2 value=false changed=true
    // t=3 value=true changed=true
    // t=4 value=true changed=false
    // t=5 value=false changed=true
    // t=6 value=false changed=false
}
