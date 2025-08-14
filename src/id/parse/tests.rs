use super::*;

#[test]
fn test_date_number() {
    assert_eq!(
        date_number(b"0412987"),
        Ok(DateNumber {
            years_since_epoch: 13,
            month: 12,
            number: NonZero::new(987).unwrap(),
            version: NonZero::new(0),
        })
    );

    assert_eq!(
        date_number(b"0001001v1"),
        Ok(DateNumber {
            years_since_epoch: 9,
            month: 1,
            number: NonZero::new(1).unwrap(),
            version: NonZero::new(1),
        })
    );

    assert!(date_number(b"0001001v").is_err());
    assert!(date_number(b"000101").is_err());
    assert!(date_number(b"9001101").is_err());
    assert!(date_number(b"9100101").is_err());
}

#[test]
fn test_version() {
    assert_eq!(version(b"123"), Ok(NonZero::new(123).unwrap()));
    assert_eq!(version(b"255"), Ok(NonZero::new(255).unwrap()));
    assert_eq!(version(b"1"), Ok(NonZero::new(1).unwrap()));

    assert_eq!(version(b""), Err(IdentifierError::InvalidVersion));
    assert_eq!(version(b"0"), Err(IdentifierError::InvalidVersion));
    assert_eq!(version(b"01"), Err(IdentifierError::InvalidVersion));
    assert_eq!(version(b"a"), Err(IdentifierError::InvalidVersion));
    assert_eq!(version(b"/"), Err(IdentifierError::InvalidVersion));
    assert_eq!(version(b" "), Err(IdentifierError::InvalidVersion));

    // we only permit version <= 255
    assert_eq!(version(b"256"), Err(IdentifierError::InvalidVersion));
    assert_eq!(version(b"257"), Err(IdentifierError::InvalidVersion));
    assert_eq!(version(b"2550"), Err(IdentifierError::InvalidVersion));
    assert_eq!(version(b"999"), Err(IdentifierError::InvalidVersion));
}

#[test]
fn test_date_old() {
    assert_eq!(date_old([b'0', b'7', b'0', b'3']), Ok((16, 3)));
    assert_eq!(date_old([b'0', b'4', b'1', b'0']), Ok((13, 10)));
    assert_eq!(date_old([b'9', b'1', b'0', b'8']), Ok((0, 8)));
    assert_eq!(date_old([b'9', b'9', b'0', b'8']), Ok((8, 8)));

    assert_eq!(
        date_old([b'0', b'7', b'0', b'4']),
        Err(IdentifierError::DateOutOfRange)
    );
    assert_eq!(
        date_old([b'9', b'1', b'0', b'7']),
        Err(IdentifierError::DateOutOfRange)
    );
    assert!(date_old([b'0', b'4', b'0', b'0']).is_err());
    assert!(date_old([b'6', b'9', b'0', b'1']).is_err());
    assert!(date_old([b'0', b'0', b'2', b'0']).is_err());
}

#[test]
fn test_date_new() {
    assert_eq!(date_new([b'1', b'4', b'1', b'2']), Ok((23, 12)));
    assert_eq!(date_new([b'0', b'7', b'0', b'4']), Ok((16, 4)));
    assert_eq!(date_new([b'0', b'8', b'0', b'4']), Ok((17, 4)));
    assert_eq!(date_new([b'0', b'0', b'0', b'1']), Ok((109, 1)));
    assert_eq!(date_new([b'0', b'7', b'0', b'3']), Ok((116, 3)));
    assert_eq!(date_new([b'0', b'1', b'0', b'1']), Ok((110, 1)));

    // check all good dates are ok
    for y1 in b'0'..=b'9' {
        for y2 in b'0'..=b'9' {
            for b in b'1'..=b'9' {
                assert!(date_new([y1, y2, b'0', b]).is_ok());
            }
            for b in b'0'..=b'2' {
                assert!(date_new([y1, y2, b'1', b]).is_ok());
            }
        }
    }

    // check bad dates are not ok
    assert!(date_new([b'0', b'0', b'0', b'0']).is_err());
    assert!(date_new([b'0', b'0', b'2', b'0']).is_err());
    assert!(date_new([b'0', b'/', b'0', b'1']).is_err());
    assert!(date_new([b'0', b'-', b'0', b'1']).is_err());
    assert!(date_new([b'/', b'0', b'0', b'1']).is_err());
    assert!(date_new([b'0', b'0', b'/', b'1']).is_err());

    for b in 0..=u8::MAX {
        if !(b'1'..=b'9').contains(&b) {
            println!("{b}");
            assert!(date_new([b'0', b'0', b'0', b]).is_err());
        }

        if !(b'0'..=b'1').contains(&b) {
            println!("{b}");
            assert!(date_new([b'0', b'0', b, b'2']).is_err());
        }

        if !(b'0'..=b'9').contains(&b) {
            println!("{b}");
            assert!(date_new([b'0', b, b'0', b'2']).is_err());
        }

        if !(b'0'..=b'9').contains(&b) {
            println!("{b}");
            assert!(date_new([b, b'0', b'0', b'2']).is_err());
        }
    }
}

#[test]
fn test_number_and_version_ok() {
    fn assert_nv_ok(len: u8, input: &[u8], number: u32, version: Option<u8>) {
        match len {
            3 => {
                assert_eq!(
                    number_and_version_len_3(input),
                    Ok((
                        NonZero::new(number as u16).unwrap(),
                        version.map(|v| NonZero::new(v).unwrap())
                    ))
                );
            }
            4 => {
                assert_eq!(
                    number_and_version_len_4(input),
                    Ok((
                        NonZero::new(number).unwrap(),
                        version.map(|v| NonZero::new(v).unwrap())
                    ))
                );
            }
            5 => {
                assert_eq!(
                    number_and_version_len_5(input),
                    Ok((
                        NonZero::new(number).unwrap(),
                        version.map(|v| NonZero::new(v).unwrap())
                    ))
                );
            }
            _ => panic!("Test called with invalid version len"),
        }
    }
    assert_nv_ok(3, b"001", 1, None);
    assert_nv_ok(3, b"999", 999, None);
    assert_nv_ok(3, b"999v1", 999, Some(1));
    assert_nv_ok(3, b"123v92", 123, Some(92));

    assert_nv_ok(4, b"0001", 1, None);
    assert_nv_ok(4, b"9999", 9999, None);
    assert_nv_ok(4, b"9999v1", 9999, Some(1));
    assert_nv_ok(4, b"0123v92", 123, Some(92));

    assert_nv_ok(5, b"00001", 1, None);
    assert_nv_ok(5, b"99999", 99999, None);
    assert_nv_ok(5, b"99999v1", 99999, Some(1));
    assert_nv_ok(5, b"01234v92", 1234, Some(92));
    assert_nv_ok(5, b"02030v8", 2030, Some(8));
}

#[test]
fn test_number_and_version_err() {
    fn assert_nv_err(len: u8, input: &[u8]) {
        match len {
            3 => assert!(number_and_version_len_3(input).is_err()),
            4 => assert!(number_and_version_len_4(input).is_err()),
            5 => assert!(number_and_version_len_5(input).is_err()),
            _ => panic!("Test called with invalid version len"),
        }
    }

    for len in [3, 4, 5] {
        assert_nv_err(len, b"000");
        assert_nv_err(len, b"/00");
        assert_nv_err(len, b"0000");
        assert_nv_err(len, b"00");
        assert_nv_err(len, b"0");
        assert_nv_err(len, b"");
        assert_nv_err(len, b"v1");
        assert_nv_err(len, b"0v1");
        assert_nv_err(len, b"100v1 ");
        assert_nv_err(len, b"v3");
        assert_nv_err(len, b"001v0");
        assert_nv_err(len, b"0001v0");
        assert_nv_err(len, b"00001v0");
        assert_nv_err(len, b"001v");
        assert_nv_err(len, b"0001v");
        assert_nv_err(len, b"00001v");
        assert_nv_err(len, b"001vc");
        assert_nv_err(len, b"001v:");
        assert_nv_err(len, b"001v/");
        assert_nv_err(len, b"001v01");
        assert_nv_err(len, b"0001vc");
        assert_nv_err(len, b"0001v:");
        assert_nv_err(len, b"0001v/");
        assert_nv_err(len, b"0001v05");
        assert_nv_err(len, b"00001vc");
        assert_nv_err(len, b"00001v:");
        assert_nv_err(len, b"00001v/");
        assert_nv_err(len, b"00001v09");
        assert_nv_err(len, b"1");
        assert_nv_err(len, b"11");
    }
}
