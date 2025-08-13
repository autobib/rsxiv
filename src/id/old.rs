use std::{fmt, num::NonZero, str::FromStr};

use super::{Identifier, IdentifierError, parse};

/// A validated old-style arxiv identifier.
///
/// An identifier is the [preferred external identifier][preferred] corresponding to an [old-style identifiers][arxiv]; that is,
/// identifiers before March 31, 2007. Note that an identifier need not correspond to an
/// actual arXiv record.
///
/// The subject class information not stored within this identifier.
///
/// [arxiv]: https://info.arxiv.org/help/arxiv_identifier.html
/// [preferred]: https://info.arxiv.org/help/arxiv_identifier_for_services.html
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct OldID {
    archive: Archive,
    years_since_epoch: u8, // this is the number of years after the earliest possible year, i.e. 1991
    month: u8,
    number: NonZero<u16>,
    version: Option<NonZero<u8>>,
}

impl OldID {
    fn from_split(archive: &[u8], date_number: &[u8]) -> Result<Self, IdentifierError> {
        let archive = parse::archive(archive)?;
        let parse::DateNumber {
            years_since_epoch,
            month,
            number,
            version,
        } = parse::date_number(date_number)?;
        Ok(Self {
            archive,
            years_since_epoch,
            month,
            number,
            version,
        })
    }

    pub fn new(
        archive: Archive,
        year: u16,
        month: u8,
        number: NonZero<u16>,
        version: Option<NonZero<u8>>,
    ) -> Result<Self, IdentifierError> {
        if !(1991..=2007).contains(&year)
            || (month == 0 || month > 12)
            || (year == 1991 && month <= 7)
            || (year == 2007 && month >= 4)
        {
            return Err(IdentifierError::DateOutOfRange);
        }

        if number.get() >= 1000 {
            return Err(IdentifierError::NumberOutOfRange);
        }

        Ok(Self {
            archive,
            years_since_epoch: (year - 1991) as u8,
            month,
            number,
            version,
        })
    }
}

impl Identifier for OldID {
    /// Return the year corresponding to the identifier. Guaranteed to land in the range
    /// `[1991..=2007]`.
    fn year(&self) -> u16 {
        1991 + (self.years_since_epoch as u16)
    }

    /// Return the month corresponding to the identifer. Guaranteed to land in the range
    /// `[1..=12]`.
    fn month(&self) -> u8 {
        self.month
    }

    /// Return the number of the identifier. Guaranteed to land in the range `[1..=999]`.
    fn number(&self) -> NonZero<u32> {
        // SAFETY: the number is initially non-zero
        unsafe { NonZero::new_unchecked(self.number.get() as u32) }
    }

    /// Return the version of the identifier. The version may not be present.
    fn version(&self) -> Option<NonZero<u8>> {
        self.version
    }
}

impl FromStr for OldID {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((arch, date_number)) => Self::from_split(arch.as_bytes(), date_number.as_bytes()),
            None => Err(IdentifierError::InvalidArchive),
        }
    }
}

impl fmt::Display for OldID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.archive.to_id())?;
        f.write_str("/")?;
        write!(
            f,
            "{:02}{:02}{:03}",
            self.month,
            self.years_since_epoch.wrapping_add(91).rem_euclid(100),
            self.number
        )?;

        if let Some(version) = self.version {
            write!(f, "v{version}")?;
        }

        Ok(())
    }
}

/// The possible archives present in an old-style arxiv identifier
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
#[repr(u8)]
pub enum Archive {
    /// Accelerator Physics
    AccPhys,
    /// Adaptation and Self-Organizing Systems
    AdapOrg,
    /// Algebraic Geometry
    AlgGeom,
    /// Atmospheric and Oceanic Physics
    AoSci,
    /// Astrophysics
    AstroPh,
    /// Atomic Physics
    AtomPh,
    /// Bayesian Analysis
    BayesAn,
    /// Chaotic Dynamics
    ChaoDyn,
    /// Chemical Physics
    ChemPh,
    /// Computation and Language
    CmpLg,
    /// Cellular Automata and Lattice Gases
    CompGas,
    /// Condensed Matter
    CondMat,
    /// Computer Science
    Cs,
    /// Differential Geometry
    DgGa,
    /// Functional Analysis
    FunctAn,
    /// General Relativity and Quantum Cosmology
    GrQc,
    /// High Energy Physics - Experiment
    HepEx,
    /// High Energy Physics - Lattice
    HepLat,
    /// High Energy Physics - Phenomenology
    HepPh,
    /// High Energy Physics - Theory
    HepTh,
    /// Mathematics,
    Math,
    /// Mathematical Physics
    MathPh,
    /// Materials Science
    MtrlTh,
    /// Nonlinear Sciences
    Nlin,
    /// Nuclear Experiment
    NuclEx,
    /// Nuclear Theory
    NuclTh,
    /// Pattern Formation and Solitons
    PattSol,
    /// Physics
    Physics,
    /// Plasma Physics
    PlasmPh,
    /// Quantum Algebra
    QAlg,
    /// Quantitative Biology
    QBio,
    /// Quantum Physics
    QuantPh,
    /// Exactly Solvable and Integrable Systems
    SolvInt,
    /// Superconductivity
    SuprCon,
}

impl Archive {
    pub fn to_id(&self) -> &'static str {
        match self {
            Archive::AccPhys => "acc-phys",
            Archive::AdapOrg => "adap-org",
            Archive::AlgGeom => "alg-geom",
            Archive::AoSci => "ao-sci",
            Archive::AstroPh => "astro-ph",
            Archive::AtomPh => "atom-ph",
            Archive::BayesAn => "bayes-an",
            Archive::ChaoDyn => "chao-dyn",
            Archive::ChemPh => "chem-ph",
            Archive::CmpLg => "cmp-lg",
            Archive::CompGas => "comp-gas",
            Archive::CondMat => "cond-mat",
            Archive::Cs => "cs",
            Archive::DgGa => "dg-ga",
            Archive::FunctAn => "funct-an",
            Archive::GrQc => "gr-qc",
            Archive::HepEx => "hep-ex",
            Archive::HepLat => "hep-lat",
            Archive::HepPh => "hep-ph",
            Archive::HepTh => "hep-th",
            Archive::Math => "math",
            Archive::MathPh => "math-ph",
            Archive::MtrlTh => "mtrl-th",
            Archive::Nlin => "nlin",
            Archive::NuclEx => "nucl-ex",
            Archive::NuclTh => "nucl-th",
            Archive::PattSol => "patt-sol",
            Archive::Physics => "physics",
            Archive::PlasmPh => "plasm-ph",
            Archive::QAlg => "q-alg",
            Archive::QBio => "q-bio",
            Archive::QuantPh => "quant-ph",
            Archive::SolvInt => "solv-int",
            Archive::SuprCon => "supr-con",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::from_id_bytes(id.as_bytes())
    }

    pub fn from_id_bytes(id: &[u8]) -> Option<Self> {
        match id {
            b"acc-phys" => Some(Archive::AccPhys),
            b"adap-org" => Some(Archive::AdapOrg),
            b"alg-geom" => Some(Archive::AlgGeom),
            b"ao-sci" => Some(Archive::AoSci),
            b"astro-ph" => Some(Archive::AstroPh),
            b"atom-ph" => Some(Archive::AtomPh),
            b"bayes-an" => Some(Archive::BayesAn),
            b"chao-dyn" => Some(Archive::ChaoDyn),
            b"chem-ph" => Some(Archive::ChemPh),
            b"cmp-lg" => Some(Archive::CmpLg),
            b"comp-gas" => Some(Archive::CompGas),
            b"cond-mat" => Some(Archive::CondMat),
            b"cs" => Some(Archive::Cs),
            b"dg-ga" => Some(Archive::DgGa),
            b"funct-an" => Some(Archive::FunctAn),
            b"gr-qc" => Some(Archive::GrQc),
            b"hep-ex" => Some(Archive::HepEx),
            b"hep-lat" => Some(Archive::HepLat),
            b"hep-ph" => Some(Archive::HepPh),
            b"hep-th" => Some(Archive::HepTh),
            b"math" => Some(Archive::Math),
            b"math-ph" => Some(Archive::MathPh),
            b"mtrl-th" => Some(Archive::MtrlTh),
            b"nlin" => Some(Archive::Nlin),
            b"nucl-ex" => Some(Archive::NuclEx),
            b"nucl-th" => Some(Archive::NuclTh),
            b"patt-sol" => Some(Archive::PattSol),
            b"physics" => Some(Archive::Physics),
            b"plasm-ph" => Some(Archive::PlasmPh),
            b"q-alg" => Some(Archive::QAlg),
            b"q-bio" => Some(Archive::QBio),
            b"quant-ph" => Some(Archive::QuantPh),
            b"solv-int" => Some(Archive::SolvInt),
            b"supr-con" => Some(Archive::SuprCon),
            _ => None,
        }
    }
}
