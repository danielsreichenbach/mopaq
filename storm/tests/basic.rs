//! Basic tests for the storm library

#[test]
fn test_format_version_values() {
    assert_eq!(storm::FormatVersion::V1 as u16, 0);
    assert_eq!(storm::FormatVersion::V2 as u16, 1);
    assert_eq!(storm::FormatVersion::V3 as u16, 2);
    assert_eq!(storm::FormatVersion::V4 as u16, 3);
}

#[test]
fn test_sector_size_calculation() {
    assert_eq!(storm::calculate_sector_size(0), 512);
    assert_eq!(storm::calculate_sector_size(3), 4096);
    assert_eq!(storm::calculate_sector_size(8), 131072);
}

#[test]
fn test_signatures() {
    assert_eq!(storm::signatures::MPQ_ARCHIVE, 0x1A51504D);
    assert_eq!(storm::signatures::MPQ_USERDATA, 0x1B51504D);
    assert_eq!(storm::signatures::HET_TABLE, 0x1A544548);
    assert_eq!(storm::signatures::BET_TABLE, 0x1A544542);
    assert_eq!(storm::signatures::STRONG_SIGNATURE, *b"NGIS");
}
