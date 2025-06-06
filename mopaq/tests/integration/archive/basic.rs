//! Basic tests for the mopaq library

#[test]
fn test_format_version_values() {
    assert_eq!(mopaq::FormatVersion::V1 as u16, 0);
    assert_eq!(mopaq::FormatVersion::V2 as u16, 1);
    assert_eq!(mopaq::FormatVersion::V3 as u16, 2);
    assert_eq!(mopaq::FormatVersion::V4 as u16, 3);
}

#[test]
fn test_sector_size_calculation() {
    assert_eq!(mopaq::calculate_sector_size(0), 512);
    assert_eq!(mopaq::calculate_sector_size(3), 4096);
    assert_eq!(mopaq::calculate_sector_size(8), 131072);
}

#[test]
fn test_signatures() {
    assert_eq!(mopaq::signatures::MPQ_ARCHIVE, 0x1A51504D);
    assert_eq!(mopaq::signatures::MPQ_USERDATA, 0x1B51504D);
    assert_eq!(mopaq::signatures::HET_TABLE, 0x1A544548);
    assert_eq!(mopaq::signatures::BET_TABLE, 0x1A544542);
    assert_eq!(mopaq::signatures::STRONG_SIGNATURE, *b"NGIS");
}
