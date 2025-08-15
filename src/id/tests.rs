use super::*;

#[test]
fn test_sort_order() {
    fn assert_strictly_increasing(lst: &[&str]) {
        let order_1 = lst.iter();
        let order_2 = lst.iter().skip(1);

        for (smaller, larger) in order_1.zip(order_2) {
            assert!(ArticleId::parse(smaller).unwrap() < ArticleId::parse(larger).unwrap());
        }
    }

    assert_strictly_increasing(&[
        "hep-th/0501001",
        "hep-th/0501002",
        "nlin/0501001",
        "nlin/0501002",
        "hep-th/0502001",
        "astro-ph/0703999v65535",
        "math/0703999v1",
        "math/0703999v65535",
        "0704.0001",
        "0704.0001v65535",
        "0704.0002",
        "0705.0001",
        "2212.99999v65535",
        "2301.00001",
        "2301.00001v1",
        "0101.00001",
        "0703.00001",
    ]);
}

#[test]
fn test_new_id() {
    fn assert_ok(id: &str, year: u16, month: u8, number: u32, version: Option<NonZero<u16>>) {
        // check the fields
        let new_id = ArticleId::from_str(id).unwrap();
        assert_eq!(new_id.year(), year);
        assert_eq!(new_id.month(), month);
        assert_eq!(new_id.number().get(), number);
        assert_eq!(new_id.version(), version);

        // check that it displays in the same way
        let displayed = new_id.to_string();
        assert_eq!(id, displayed);
        assert!(displayed.len() <= MAX_ID_FORMATTED_LEN);

        // check that it is equal to constructing from parameters
        assert_eq!(
            Ok(new_id),
            ArticleId::new(year, month, None, NonZero::new(number).unwrap(), version)
        );

        let ser = new_id.serialize();
        // check round-trip (de)serialization
        assert_eq!(ArticleId::deserialize(ser), Some(new_id));

        // check correctness of bitmask
        assert_eq!(ser, ser & SERIALIZED_BITMASK);
    }
    assert_ok("1304.0567", 2013, 4, 567, None);
    assert_ok("1304.0001v12", 2013, 4, 1, NonZero::new(12));
    assert_ok("0704.0001v1", 2007, 4, 1, NonZero::new(1));
    assert_ok("1412.7878", 2014, 12, 7878, None);
    assert_ok("1501.00001", 2015, 1, 1, None);
    assert_ok("0703.99999v255", 2107, 3, 99999, NonZero::new(255));
    assert_ok("0001.00001", 2100, 1, 1, None);

    assert!(ArticleId::from_str("").is_err());
    assert!(ArticleId::from_str("13").is_err());
    assert!(ArticleId::from_str("0703.9999").is_err());
    assert!(ArticleId::from_str("0703.99999v").is_err());
    assert!(ArticleId::from_str("0703.99999v0").is_err());
    assert!(ArticleId::from_str("0704.99999").is_err());
}

#[test]
fn test_old_id() {
    fn assert_fields(
        id: ArticleId,
        archive: Archive,
        year: u16,
        month: u8,
        number: u32,
        version: Option<NonZero<u16>>,
    ) {
        println!("{id}");
        // check the fields
        assert_eq!(id.archive(), Some(archive));
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
        number: u32,
        version: Option<NonZero<u16>>,
    ) {
        let old_id = ArticleId::from_str(id).unwrap();

        assert_fields(old_id, archive, year, month, number, version);

        // check that it displays in the same way
        let displayed = old_id.to_string();
        assert_eq!(id, displayed);
        assert!(displayed.len() <= MAX_ID_FORMATTED_LEN);

        let ser = old_id.serialize();
        // check round-trip (de)serialization
        assert_eq!(ArticleId::deserialize(ser), Some(old_id));

        // check correctness of bitmask
        assert_eq!(ser, ser & SERIALIZED_BITMASK);

        // check that it is equal to constructing from parameters
        assert_eq!(
            Ok(old_id),
            ArticleId::new(
                year,
                month,
                Some(archive),
                NonZero::new(number).unwrap(),
                version
            )
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
    assert_ok("supr-con/0001001", Archive::SuprCon, 2000, 1, 1, None);
    assert_ok("acc-phys/0001001", Archive::AccPhys, 2000, 1, 1, None);
    assert_ok(
        "acc-phys/0001001v10000",
        Archive::AccPhys,
        2000,
        1,
        1,
        NonZero::new(10000),
    );

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

    assert!(ArticleId::from_str("nlin.Z/0101010v1").is_err());
    assert!(ArticleId::from_str("nlin.zz/0101010v1").is_err());
    assert!(ArticleId::from_str("nlin./0101010v1").is_err());
    assert!(ArticleId::from_str("./0101010v1").is_err());
    assert!(ArticleId::from_str("a./0101010v1").is_err());
    assert!(ArticleId::from_str("a/0101010v1").is_err());
    assert!(ArticleId::from_str("a\\0101010v1").is_err());
    assert!(ArticleId::from_str("a.0101010v1").is_err());
    assert!(ArticleId::from_str("0101010v1").is_err());

    assert!(ArticleId::from_str("").is_err());
    assert!(ArticleId::from_str("nlin.ZZ/0101010v0").is_err());
    assert!(ArticleId::from_str("bad/0101010v0").is_err());

    assert!(ArticleId::from_str("hep-lat.ZZ/9108001").is_ok());
    assert!(ArticleId::from_str("hep-lat.ZZ/9107001").is_err());
    assert!(ArticleId::from_str("hep-lat/9107001").is_err());
    assert!(ArticleId::from_str("hep-lat.ZZ/910801").is_err());
    assert!(ArticleId::from_str("hep-lat.ZZ/9108000").is_err());
    assert!(ArticleId::from_str("hep-lat/910801").is_err());
    assert!(ArticleId::from_str("hep-lat/9108000").is_err());
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
