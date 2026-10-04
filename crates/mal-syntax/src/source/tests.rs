use super::*;

#[test]
fn locations_are_one_based_and_unicode_aware() {
    let source = SourceFile::new(FileId::new(3), "sample.mal", "aé\nxyz".into());

    assert_eq!(source.location(0), Some(Location { line: 1, column: 1 }));
    assert_eq!(source.location(3), Some(Location { line: 1, column: 3 }));
    assert_eq!(source.location(4), Some(Location { line: 2, column: 1 }));
    assert_eq!(source.location(5), Some(Location { line: 2, column: 2 }));
    assert_eq!(source.location(2), None);
}

#[test]
fn lines_exclude_line_endings() {
    let source = SourceFile::new(
        FileId::new(0),
        "sample.mal",
        "first\r\nsecond\nthird\rfourth".into(),
    );

    assert_eq!(source.line(1).expect("first line").text, "first");
    assert_eq!(source.line(2).expect("second line").text, "second");
    assert_eq!(source.line(3).expect("third line").text, "third");
    assert_eq!(source.line(4).expect("fourth line").text, "fourth");
    assert!(source.line(5).is_none());
    assert_eq!(source.location(20), Some(Location { line: 4, column: 1 }));
}

#[test]
fn span_membership_checks_file_bounds_and_source_positions() {
    let source = SourceFile::new(FileId::new(1), "sample.mal", "é\r\n".into());

    assert!(source.contains(Span::new(FileId::new(1), 0, 2)));
    assert!(!source.contains(Span::new(FileId::new(2), 0, 2)));
    assert!(!source.contains(Span::new(FileId::new(1), 0, 1)));
    assert!(!source.contains(Span::new(FileId::new(1), 3, 3)));
    assert_eq!(source.location(3), None);
}

#[test]
fn converts_between_byte_offsets_and_zero_based_utf16_positions() {
    let source = SourceFile::new(FileId::new(0), "sample.mal", "a😀\r\né\n".into());

    assert_eq!(
        source.utf16_position(5),
        Some(Utf16Position {
            line: 0,
            character: 3,
        })
    );
    assert_eq!(
        source.utf16_position(7),
        Some(Utf16Position {
            line: 1,
            character: 0,
        })
    );
    assert_eq!(
        source.byte_offset_utf16(Utf16Position {
            line: 0,
            character: 3,
        }),
        Some(5)
    );
    assert_eq!(
        source.byte_offset_utf16(Utf16Position {
            line: 1,
            character: 1,
        }),
        Some(9)
    );
}

#[test]
fn rejects_positions_inside_utf8_or_utf16_characters_and_line_endings() {
    let source = SourceFile::new(FileId::new(0), "sample.mal", "a😀\r\n".into());

    assert_eq!(source.utf16_position(2), None);
    assert_eq!(source.utf16_position(6), None);
    assert_eq!(
        source.byte_offset_utf16(Utf16Position {
            line: 0,
            character: 2,
        }),
        None
    );
    assert_eq!(
        source.byte_offset_utf16(Utf16Position {
            line: 3,
            character: 0,
        }),
        None
    );
}
