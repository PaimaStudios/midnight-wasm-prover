pub(crate) fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub(crate) const EXPECTED_DATA: &[(&str, [u8; 32], &str)] = &[
    (
        "bls_midnight_2p10",
        hexhash(b"46b2290933cbed4c378889e4ba971f1a92888331ffb09466acd4ff61a1e2cb42"),
        "public parameters for k=10",
    ),
    (
        "bls_midnight_2p11",
        hexhash(b"9901589d7956ff58be0d85569b2f455b77b58c3758026ffb5bbe4807000b96d1"),
        "public parameters for k=11",
    ),
    (
        "bls_midnight_2p12",
        hexhash(b"ef08eb3fcf62df8f72c515cffa027e681808b530cb016eea104115545ef6d5c8"),
        "public parameters for k=12",
    ),
    (
        "bls_midnight_2p13",
        hexhash(b"d3324910969c4cc54143b8045b649e5c3a4bd5fb7b8f85fe1b770f640ce1c803"),
        "public parameters for k=13",
    ),
    (
        "bls_midnight_2p14",
        hexhash(b"fc253016885ec830e97808c9ec920bb5cab5c21af590380a6cb5eb0538e2b244"),
        "public parameters for k=14",
    ),
    (
        "bls_midnight_2p15",
        hexhash(b"724c7c3d779148bb113c7ee9c034b2f27db16e6bdf315fde90105a9bad00b1de"),
        "public parameters for k=15",
    ),
    (
        "bls_midnight_2p16",
        hexhash(b"09c877216d6589b370263e18af40a030a901b41a7a7c37ef58c9901db41f05c6"),
        "public parameters for k=16",
    ),
    (
        "bls_midnight_2p17",
        hexhash(b"4a9ef6c7c0619aab74eede44b13e753e3ba54508a02dd3b7106a949aabb73b74"),
        "public parameters for k=17",
    ),
    (
        "bls_filecoin_2p18",
        hexhash(b"4f023825c14cc0a88070c70588a932519186d646094eddbff93c87a46060fd28"),
        "public parameters for k=18",
    ),
    (
        "bls_filecoin_2p19",
        hexhash(b"0574a536c128142e89c0f28198d048145e2bb2bf645c8b81c8697cba445a1fb1"),
        "public parameters for k=19",
    ),
    (
        "bls_filecoin_2p20",
        hexhash(b"75a1774fdf0848f4ff82790202e5c1401598bafea27321b77180d96c56e62228"),
        "public parameters for k=20",
    ),
    (
        "bls_filecoin_2p21",
        hexhash(b"e05fcbe4f7692800431cfc32e972be629c641fca891017be09a8384d0b5f8d3c"),
        "public parameters for k=21",
    ),
    (
        "bls_filecoin_2p22",
        hexhash(b"277d9c8140c02a1d4472d5da65a823fc883bc4596e69734fb16ca463d193186b"),
        "public parameters for k=22",
    ),
    (
        "bls_filecoin_2p23",
        hexhash(b"7b8dc4b2e809ef24ed459cabaf9286774cf63f2e6e2086f0d9fb014814bdfc97"),
        "public parameters for k=23",
    ),
    (
        "bls_filecoin_2p24",
        hexhash(b"e6b02dccf381a5fc7a79ba4d87612015eba904241f81521e2dea39a60ab6b812"),
        "public parameters for k=24",
    ),
];

/// Parse a 256-bit hex hash at const time.
pub const fn hexhash(hex: &[u8]) -> [u8; 32] {
    match const_hex::const_decode_to_array(hex) {
        Ok(hash) => hash,
        Err(_) => panic!("hash should be correct format"),
    }
}
