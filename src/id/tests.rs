use super::*;

#[test]
fn test_new_id() {
    fn assert_ok(id: &str, year: u16, month: u8, number: u32, version: Option<NonZero<u8>>) {
        // check the fields
        let new_id = NewId::from_str(id).unwrap();
        assert_eq!(new_id.year(), year);
        assert_eq!(new_id.month(), month);
        assert_eq!(new_id.number().get(), number);
        assert_eq!(new_id.version(), version);

        // check that it displays in the same way
        assert_eq!(id, new_id.to_string());

        // check that it is equal to constructing from parameters
        assert_eq!(
            Ok(new_id),
            NewId::new(year, month, NonZero::new(number).unwrap(), version)
        );

        // check the ArticleId variants as well
        let art_id = ArticleId::from_str(id).unwrap();
        assert_eq!(ArticleId::New(new_id), art_id);
        assert_eq!(id, art_id.to_string());
    }
    assert_ok("1304.0567", 2013, 4, 567, None);
    assert_ok("1304.0001v12", 2013, 4, 1, NonZero::new(12));
    assert_ok("0704.0001v1", 2007, 4, 1, NonZero::new(1));
    assert_ok("1412.7878", 2014, 12, 7878, None);
    assert_ok("1501.00001", 2015, 1, 1, None);
    assert_ok("0703.99999v255", 2107, 3, 99999, NonZero::new(255));
    assert_ok("0001.00001", 2100, 1, 1, None);

    assert!(NewId::from_str("").is_err());
    assert!(NewId::from_str("13").is_err());
    assert!(NewId::from_str("0703.9999").is_err());
    assert!(NewId::from_str("0703.99999v").is_err());
    assert!(NewId::from_str("0703.99999v0").is_err());
    assert!(NewId::from_str("0704.99999").is_err());
}

#[test]
fn test_old_id() {
    fn assert_fields(
        id: OldId,
        archive: Archive,
        year: u16,
        month: u8,
        number: u16,
        version: Option<NonZero<u8>>,
    ) {
        // check the fields
        assert_eq!(id.archive(), archive);
        assert_eq!(id.year(), year);
        assert_eq!(id.month(), month);
        assert_eq!(id.number().get(), number.into());
        assert_eq!(id.version(), version);
    }

    fn assert_ok(
        id: &str,
        archive: Archive,
        year: u16,
        month: u8,
        number: u16,
        version: Option<NonZero<u8>>,
    ) {
        let old_id = OldId::from_str(id).unwrap();

        assert_fields(old_id, archive, year, month, number, version);

        // check that it displays in the same way
        assert_eq!(id, old_id.to_string());

        // check that it is equal to constructing from parameters
        assert_eq!(
            Ok(old_id),
            OldId::new(archive, year, month, NonZero::new(number).unwrap(), version)
        );
    }

    assert_ok("math/9205123", Archive::Math, 1992, 5, 123, None);
    assert_ok(
        "hep-lat/9108001v1",
        Archive::HepLat,
        1991,
        8,
        1,
        NonZero::new(1),
    );
    assert_ok(
        "cond-mat/0703999v2",
        Archive::CondMat,
        2007,
        3,
        999,
        NonZero::new(2),
    );
    assert_ok("gr-qc/0703001", Archive::GrQc, 2007, 3, 1, None);
    assert_ok("chao-dyn/0012001", Archive::ChaoDyn, 2000, 12, 1, None);

    // check that the subject class is pruned correctly
    assert_fields(
        "math.CA/9310001".parse().unwrap(),
        Archive::Math,
        1993,
        10,
        1,
        None,
    );
    assert_fields(
        "nlin.ZZ/0101010v1".parse().unwrap(),
        Archive::Nlin,
        2001,
        1,
        10,
        NonZero::new(1),
    );

    assert!(OldId::from_str("nlin.Z/0101010v1").is_err());
    assert!(OldId::from_str("nlin.zz/0101010v1").is_err());
    assert!(OldId::from_str("nlin./0101010v1").is_err());
    assert!(OldId::from_str("./0101010v1").is_err());
    assert!(OldId::from_str("a./0101010v1").is_err());
    assert!(OldId::from_str("a/0101010v1").is_err());
    assert!(OldId::from_str("a\\0101010v1").is_err());
    assert!(OldId::from_str("a.0101010v1").is_err());
    assert!(OldId::from_str("0101010v1").is_err());

    assert!(OldId::from_str("").is_err());
    assert!(OldId::from_str("nlin.ZZ/0101010v0").is_err());
    assert!(OldId::from_str("bad/0101010v0").is_err());

    assert!(OldId::from_str("hep-lat.ZZ/9108001").is_ok());
    assert!(OldId::from_str("hep-lat.ZZ/9107001").is_err());
    assert!(OldId::from_str("hep-lat/9107001").is_err());
    assert!(OldId::from_str("hep-lat.ZZ/910801").is_err());
    assert!(OldId::from_str("hep-lat.ZZ/9108000").is_err());
    assert!(OldId::from_str("hep-lat/910801").is_err());
    assert!(OldId::from_str("hep-lat/9108000").is_err());
}

#[test]
fn test_archive() {
    use Archive::*;
    for variant in [
        AccPhys, AdapOrg, AlgGeom, AoSci, AstroPh, AtomPh, BayesAn, ChaoDyn, ChemPh, CmpLg,
        CompGas, CondMat, Cs, DgGa, FunctAn, GrQc, HepEx, HepLat, HepPh, HepTh, Math, MathPh,
        MtrlTh, Nlin, NuclEx, NuclTh, PattSol, Physics, PlasmPh, QAlg, QBio, QuantPh, SolvInt,
        SuprCon,
    ] {
        assert_eq!(Archive::from_id(variant.to_id()), Some(variant));
    }

    assert!(Archive::from_id("").is_none());
    assert!(Archive::from_id("nuclex").is_none());
    assert!(Archive::from_id(" ").is_none());
    assert!(Archive::from_id(" math").is_none());
    assert!(Archive::from_id("ma").is_none());
}
