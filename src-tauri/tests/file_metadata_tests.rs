use chrono::{DateTime, TimeZone, Utc};
use reqwest_dav::list_cmd::ListFile;
use uniclipboard::file_metadata::FileMetadata;

fn create_list_file(
    href: &str,
    content_length: i64,
    last_modified: DateTime<Utc>,
    content_type: &str,
    tag: Option<String>,
) -> ListFile {
    ListFile {
        href: href.to_string(),
        content_length,
        last_modified,
        content_type: content_type.to_string(),
        tag,
    }
}

#[test]
fn test_file_metadata_from_list_file() {
    let host = "https://example.com/dav";
    let list_file = create_list_file(
        "/dav/test/path/test_file.txt",
        100,
        Utc.with_ymd_and_hms(2023, 5, 1, 12, 0, 0).unwrap(),
        "text/plain",
        Some("etag123".to_string()),
    );

    let metadata = FileMetadata::from_list_file(&list_file, host);

    assert_eq!(metadata.name, "test_file.txt");
    assert_eq!(metadata.dir, "/test/path");
    assert_eq!(metadata.size, 100);
    assert_eq!(
        metadata.last_modified,
        Utc.with_ymd_and_hms(2023, 5, 1, 12, 0, 0).unwrap()
    );
    assert_eq!(metadata.content_type, "text/plain");
    assert_eq!(metadata.tag, Some("etag123".to_string()));
}

#[test]
fn test_file_metadata_get_path() {
    let host = "https://example.com/dav";
    let list_file = create_list_file(
        "/dav/test/path/test_file.txt",
        100,
        Utc::now(),
        "text/plain",
        None,
    );
    let metadata = FileMetadata::from_list_file(&list_file, host);

    assert_eq!(metadata.get_path(), "/test/path/test_file.txt");
}

#[test]
fn test_file_metadata_get_device_id() {
    let host = "https://example.com/dav";
    let list_file = create_list_file(
        "/dav/test/path/device123_uuid456.json",
        100,
        Utc::now(),
        "application/json",
        None,
    );
    let metadata = FileMetadata::from_list_file(&list_file, host);

    assert_eq!(metadata.get_device_id(), "device123");
}

#[test]
fn test_file_metadata_is_newer_than() {
    let host = "https://example.com/dav";
    let older = FileMetadata::from_list_file(
        &create_list_file(
            "/dav/test/older.txt",
            100,
            Utc.with_ymd_and_hms(2023, 5, 1, 12, 0, 0).unwrap(),
            "text/plain",
            None,
        ),
        host,
    );
    let newer = FileMetadata::from_list_file(
        &create_list_file(
            "/dav/test/newer.txt",
            100,
            Utc.with_ymd_and_hms(2023, 5, 2, 12, 0, 0).unwrap(),
            "text/plain",
            None,
        ),
        host,
    );

    assert!(newer.is_newer_than(&older));
    assert!(!older.is_newer_than(&newer));
}

#[test]
fn test_file_metadata_get_prefix() {
    assert_eq!(
        FileMetadata::get_prefix("https://example.com/prefix/path"),
        Some("/prefix".to_string())
    );
    assert_eq!(FileMetadata::get_prefix("https://example.com"), None);
}
