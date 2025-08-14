/// The possible archives present in an old-style arxiv identifier
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
    /// Convert to a raw identifier, as used internally by arXiv.
    ///
    /// The raw identifier is the enum variant name in kebab-case.
    /// ```
    /// use rsxiv::id::Archive;
    /// assert_eq!(Archive::QuantPh.to_id(), "quant-ph");
    /// ```
    #[must_use]
    pub const fn to_id(&self) -> &'static str {
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

    /// Read from a raw identifier.
    ///
    /// The raw identifier is the enum variant name in kebab-case.
    /// ```
    /// use rsxiv::id::Archive;
    /// assert_eq!(Archive::from_id("math"), Some(Archive::Math));
    /// ```
    /// The identifier must match exactly, or this will fail.
    /// ```
    /// # use rsxiv::id::Archive;
    /// assert_eq!(Archive::from_id("solv-int "), None);
    /// ```
    #[must_use]
    pub const fn from_id(id: &str) -> Option<Self> {
        Self::from_id_bytes(id.as_bytes())
    }

    /// Read from a raw identifier as bytes.
    ///
    /// The raw identifier is the enum variant name in kebab-case.
    /// ```
    /// use rsxiv::id::Archive;
    /// assert_eq!(Archive::from_id_bytes(b"supr-con"), Some(Archive::SuprCon));
    /// ```
    #[must_use]
    pub const fn from_id_bytes(id: &[u8]) -> Option<Self> {
        match strip_prefix(id) {
            Some((archive, b"")) => Some(archive),
            _ => None,
        }
    }
}

/// Strip a valid archive prefix from a `&[u8]`, returning the matched archive and trailing character.
///
/// This is implemented as a large match table so the compiler can optimize the lookup against the
/// character sets. This also makes this method a `const fn`.
#[inline]
pub const fn strip_prefix(s: &[u8]) -> Option<(Archive, &[u8])> {
    match s {
        [b'a', b'c', b'c', b'-', b'p', b'h', b'y', b's', t @ ..] => Some((Archive::AccPhys, t)),
        [b'a', b'd', b'a', b'p', b'-', b'o', b'r', b'g', t @ ..] => Some((Archive::AdapOrg, t)),
        [b'a', b'l', b'g', b'-', b'g', b'e', b'o', b'm', t @ ..] => Some((Archive::AlgGeom, t)),
        [b'a', b'o', b'-', b's', b'c', b'i', t @ ..] => Some((Archive::AoSci, t)),
        [b'a', b's', b't', b'r', b'o', b'-', b'p', b'h', t @ ..] => Some((Archive::AstroPh, t)),
        [b'a', b't', b'o', b'm', b'-', b'p', b'h', t @ ..] => Some((Archive::AtomPh, t)),
        [b'b', b'a', b'y', b'e', b's', b'-', b'a', b'n', t @ ..] => Some((Archive::BayesAn, t)),
        [b'c', b'h', b'a', b'o', b'-', b'd', b'y', b'n', t @ ..] => Some((Archive::ChaoDyn, t)),
        [b'c', b'h', b'e', b'm', b'-', b'p', b'h', t @ ..] => Some((Archive::ChemPh, t)),
        [b'c', b'm', b'p', b'-', b'l', b'g', t @ ..] => Some((Archive::CmpLg, t)),
        [b'c', b'o', b'm', b'p', b'-', b'g', b'a', b's', t @ ..] => Some((Archive::CompGas, t)),
        [b'c', b'o', b'n', b'd', b'-', b'm', b'a', b't', t @ ..] => Some((Archive::CondMat, t)),
        [b'c', b's', t @ ..] => Some((Archive::Cs, t)),
        [b'd', b'g', b'-', b'g', b'a', t @ ..] => Some((Archive::DgGa, t)),
        [b'f', b'u', b'n', b'c', b't', b'-', b'a', b'n', t @ ..] => Some((Archive::FunctAn, t)),
        [b'g', b'r', b'-', b'q', b'c', t @ ..] => Some((Archive::GrQc, t)),
        [b'h', b'e', b'p', b'-', b'e', b'x', t @ ..] => Some((Archive::HepEx, t)),
        [b'h', b'e', b'p', b'-', b'l', b'a', b't', t @ ..] => Some((Archive::HepLat, t)),
        [b'h', b'e', b'p', b'-', b'p', b'h', t @ ..] => Some((Archive::HepPh, t)),
        [b'h', b'e', b'p', b'-', b't', b'h', t @ ..] => Some((Archive::HepTh, t)),
        [b'm', b'a', b't', b'h', b'-', b'p', b'h', t @ ..] => Some((Archive::MathPh, t)),
        [b'm', b'a', b't', b'h', t @ ..] => Some((Archive::Math, t)),
        [b'm', b't', b'r', b'l', b'-', b't', b'h', t @ ..] => Some((Archive::MtrlTh, t)),
        [b'n', b'l', b'i', b'n', t @ ..] => Some((Archive::Nlin, t)),
        [b'n', b'u', b'c', b'l', b'-', b'e', b'x', t @ ..] => Some((Archive::NuclEx, t)),
        [b'n', b'u', b'c', b'l', b'-', b't', b'h', t @ ..] => Some((Archive::NuclTh, t)),
        [b'p', b'a', b't', b't', b'-', b's', b'o', b'l', t @ ..] => Some((Archive::PattSol, t)),
        [b'p', b'h', b'y', b's', b'i', b'c', b's', t @ ..] => Some((Archive::Physics, t)),
        [b'p', b'l', b'a', b's', b'm', b'-', b'p', b'h', t @ ..] => Some((Archive::PlasmPh, t)),
        [b'q', b'-', b'a', b'l', b'g', t @ ..] => Some((Archive::QAlg, t)),
        [b'q', b'-', b'b', b'i', b'o', t @ ..] => Some((Archive::QBio, t)),
        [b'q', b'u', b'a', b'n', b't', b'-', b'p', b'h', t @ ..] => Some((Archive::QuantPh, t)),
        [b's', b'o', b'l', b'v', b'-', b'i', b'n', b't', t @ ..] => Some((Archive::SolvInt, t)),
        [b's', b'u', b'p', b'r', b'-', b'c', b'o', b'n', t @ ..] => Some((Archive::SuprCon, t)),
        _ => None,
    }
}
