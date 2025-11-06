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
        "bls_filecoin_2p10",
        hexhash(b"d1a3403c1f8669e82ed28d9391e13011aea76801b28fe14b42bf76d141b4efa2"),
        "public parameters for k=10",
    ),
    (
        "bls_filecoin_2p11",
        hexhash(b"b5047f05800dbd84fd1ea43b96a8850e128b7a595ed132cd72588cc2cb146b29"),
        "public parameters for k=11",
    ),
    (
        "bls_filecoin_2p12",
        hexhash(b"b32791775af5fff1ae5ead682c3d8832917ebb0652b43cf810a1e3956eb27a71"),
        "public parameters for k=12",
    ),
    (
        "bls_filecoin_2p13",
        hexhash(b"b9af43892c3cb90321fa00a36e5e59051f356df145d7f58368531f28d212937b"),
        "public parameters for k=13",
    ),
    (
        "bls_filecoin_2p14",
        hexhash(b"4923e5a7fbb715d81cdb5c03b9c0e211768d35ccc52d82f49c3d93bcf8d36a56"),
        "public parameters for k=14",
    ),
    (
        "bls_filecoin_2p15",
        hexhash(b"162fac0cf70b9b02e02195ec37013c04997b39dc1831a97d5a83f47a9ce39c97"),
        "public parameters for k=15",
    ),
    (
        "bls_filecoin_2p16",
        hexhash(b"4ebc0d077fe6645e9b7ca6563217be2176f00dfe39cc97b3f60ecbad3573f973"),
        "public parameters for k=16",
    ),
    (
        "bls_filecoin_2p17",
        hexhash(b"7228c4519e96ece2c54bf2f537d9f26b0ed042819733726623fab5e17eac4360"),
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
