//! The viewing-format catalog: the single source of truth for the format
//! hierarchy, its labels, and which projection × technology × auditorium
//! combinations are legal.
//!
//! Both apps render their format pickers from [`format_catalog`], so labels
//! and legality rules cannot drift between the Rust validation, the desktop
//! UI, and the Android UI.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::domain::{Dci, Film, Flag, Format, HomeMedium, Imax, Projection, ScreenClass};

/// A selectable catalog entry: a stable key and its display label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormatOption {
    /// The key used in a serialized [`Format`] (and as the enum's serde tag).
    pub key: String,
    /// The human-readable label.
    pub label: String,
}

impl FormatOption {
    fn new(key: &str, label: &str) -> Self {
        Self {
            key: key.to_owned(),
            label: label.to_owned(),
        }
    }
}

/// One legal projection × technology × auditorium combination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormatLegality {
    /// The projection family: `dci`, `imax`, or `film`.
    pub family: String,
    /// The technology key.
    pub technology: String,
    /// The auditorium key.
    pub screen: String,
    /// The flags this combination may carry.
    pub allowed_flags: Vec<String>,
    /// The flags this combination must carry.
    pub forced_flags: Vec<String>,
}

/// The whole format catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormatCatalog {
    /// The projection families, in display order.
    pub families: Vec<FormatOption>,
    /// The technologies per family key.
    pub technologies: BTreeMap<String, Vec<FormatOption>>,
    /// The auditoria legal for each family key.
    pub screens: BTreeMap<String, Vec<FormatOption>>,
    /// The home release mediums.
    pub mediums: Vec<FormatOption>,
    /// The presentation flags.
    pub flag_labels: Vec<FormatOption>,
    /// Every legal projection × technology × auditorium combination.
    pub legal: Vec<FormatLegality>,
}

/// The format keys of the presentation flags, in display order.
const FLAG_KEYS: [(&str, Flag); 3] = [
    ("three_d", Flag::ThreeD),
    ("hfr", Flag::Hfr),
    ("dbox", Flag::Dbox),
];

impl Dci {
    /// The catalog key for this technology.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            Dci::Lamp2K => "2k_lamp",
            Dci::Laser2K => "2k_laser",
            Dci::Lamp4K => "4k_lamp",
            Dci::Laser4K => "4k_laser",
            Dci::Custom(id) => id,
        }
    }
}

impl Imax {
    /// The catalog key for this technology.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            Imax::Xenon2K => "2k_xenon",
            Imax::Laser4KSingle => "4k_single_laser",
            Imax::Laser4KDual => "4k_dual_laser",
            Imax::Film70mm => "70mm",
            Imax::Custom(id) => id,
        }
    }
}

impl Film {
    /// The catalog key for this technology.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            Film::Mm35 => "35mm",
            Film::Mm70 => "70mm",
            Film::Custom(id) => id,
        }
    }
}

impl ScreenClass {
    /// The catalog key for this auditorium class.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            ScreenClass::Regular => "regular",
            ScreenClass::Plf => "plf",
            ScreenClass::Vip => "vip",
            ScreenClass::FourDx => "4dx",
            ScreenClass::ScreenX => "screenx",
            ScreenClass::ImaxClassic => "imax_class",
            ScreenClass::ImaxRetrofit => "imax_retrofit",
            ScreenClass::Ultra4Dx => "ultra_4dx",
            ScreenClass::Custom(id) => id,
        }
    }
}

impl HomeMedium {
    /// The catalog key for this release medium.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            HomeMedium::Dvd => "dvd",
            HomeMedium::HdDvd => "hd_dvd",
            HomeMedium::BluRay => "bluray",
            HomeMedium::UhdBluRay => "uhd_bluray",
            HomeMedium::LaserDisc => "laserdisc",
            HomeMedium::Streaming => "streaming",
            HomeMedium::Broadcast => "broadcast",
            HomeMedium::Telecine => "telecine",
            HomeMedium::Custom(id) => id,
        }
    }
}

impl Projection {
    /// The family key: `dci`, `imax`, or `film`.
    #[must_use]
    pub fn family_key(&self) -> &'static str {
        match self {
            Projection::Dci(_) => "dci",
            Projection::Imax(_) => "imax",
            Projection::Film(_) => "film",
        }
    }

    /// The technology key within the family.
    #[must_use]
    pub fn tech_key(&self) -> &str {
        match self {
            Projection::Dci(dci) => dci.key(),
            Projection::Imax(imax) => imax.key(),
            Projection::Film(film) => film.key(),
        }
    }
}

/// The family labels, in display order.
const FAMILY_LABELS: [(&str, &str); 3] =
    [("dci", "Digital (DCI)"), ("imax", "IMAX"), ("film", "Film")];

/// The DCI technology options.
fn dci_options() -> Vec<FormatOption> {
    vec![
        FormatOption::new("2k_lamp", "2K xenon"),
        FormatOption::new("2k_laser", "2K laser"),
        FormatOption::new("4k_lamp", "4K xenon"),
        FormatOption::new("4k_laser", "4K laser"),
    ]
}

/// The IMAX technology options.
fn imax_options() -> Vec<FormatOption> {
    vec![
        FormatOption::new("2k_xenon", "2K xenon"),
        FormatOption::new("4k_single_laser", "4K single laser"),
        FormatOption::new("4k_dual_laser", "4K dual laser"),
        FormatOption::new("70mm", "70mm"),
    ]
}

/// The film technology options.
fn film_options() -> Vec<FormatOption> {
    vec![
        FormatOption::new("35mm", "35mm"),
        FormatOption::new("70mm", "70mm"),
    ]
}

/// Every auditorium class, in display order.
fn all_screens() -> Vec<(ScreenClass, &'static str)> {
    vec![
        (ScreenClass::Regular, "Regular"),
        (ScreenClass::Plf, "PLF"),
        (ScreenClass::Vip, "VIP"),
        (ScreenClass::FourDx, "4DX"),
        (ScreenClass::ScreenX, "ScreenX"),
        (ScreenClass::ImaxClassic, "IMAX 1.43:1"),
        (ScreenClass::ImaxRetrofit, "IMAX 1.90:1"),
        (ScreenClass::Ultra4Dx, "Ultra 4DX"),
    ]
}

/// A representative projection for a family, used to filter its auditoria
/// (every technology in a family runs on the same screens).
fn representative(family: &str) -> Projection {
    match family {
        "imax" => Projection::Imax(Imax::Laser4KDual),
        "film" => Projection::Film(Film::Mm35),
        _ => Projection::Dci(Dci::Laser4K),
    }
}

/// The technologies of a family as concrete projections.
fn technologies(family: &str) -> Vec<Projection> {
    match family {
        "imax" => vec![
            Projection::Imax(Imax::Xenon2K),
            Projection::Imax(Imax::Laser4KSingle),
            Projection::Imax(Imax::Laser4KDual),
            Projection::Imax(Imax::Film70mm),
        ],
        "film" => vec![Projection::Film(Film::Mm35), Projection::Film(Film::Mm70)],
        _ => vec![
            Projection::Dci(Dci::Lamp2K),
            Projection::Dci(Dci::Laser2K),
            Projection::Dci(Dci::Lamp4K),
            Projection::Dci(Dci::Laser4K),
        ],
    }
}

/// The technology options of a family.
fn technology_options(family: &str) -> Vec<FormatOption> {
    match family {
        "imax" => imax_options(),
        "film" => film_options(),
        _ => dci_options(),
    }
}

/// The auditoria a family's projections can run in.
fn family_screens(family: &str) -> Vec<(ScreenClass, &'static str)> {
    let projection = representative(family);
    all_screens()
        .into_iter()
        .filter(|(screen, _)| projection.runs_on(screen))
        .collect()
}

/// The presentation-flag keys allowed for a projection and auditorium.
fn allowed_flags(projection: &Projection, screen: &ScreenClass) -> Vec<String> {
    FLAG_KEYS
        .iter()
        .filter(|(_, flag)| projection.allows(*flag) && screen.allows(*flag))
        .map(|(key, _)| (*key).to_owned())
        .collect()
}

/// The presentation-flag keys the auditorium forces.
fn forced_flags(screen: &ScreenClass) -> Vec<String> {
    FLAG_KEYS
        .iter()
        .filter(|(_, flag)| screen.forces(*flag))
        .map(|(key, _)| (*key).to_owned())
        .collect()
}

/// The whole format catalog, built from the non-custom variants.
#[must_use]
pub fn format_catalog() -> FormatCatalog {
    let mut technologies_map = BTreeMap::new();
    let mut screens_map = BTreeMap::new();
    let mut legal = Vec::new();
    for (family, _) in FAMILY_LABELS {
        technologies_map.insert(family.to_owned(), technology_options(family));
        screens_map.insert(
            family.to_owned(),
            family_screens(family)
                .into_iter()
                .map(|(screen, label)| FormatOption::new(screen.key(), label))
                .collect(),
        );
        for projection in technologies(family) {
            for (screen, _) in family_screens(family) {
                legal.push(FormatLegality {
                    family: family.to_owned(),
                    technology: projection.tech_key().to_owned(),
                    screen: screen.key().to_owned(),
                    allowed_flags: allowed_flags(&projection, &screen),
                    forced_flags: forced_flags(&screen),
                });
            }
        }
    }
    FormatCatalog {
        families: FAMILY_LABELS
            .iter()
            .map(|(key, label)| FormatOption::new(key, label))
            .collect(),
        technologies: technologies_map,
        screens: screens_map,
        mediums: vec![
            FormatOption::new("dvd", "DVD"),
            FormatOption::new("hd_dvd", "HD DVD"),
            FormatOption::new("bluray", "Blu-ray"),
            FormatOption::new("uhd_bluray", "4K Ultra HD Blu-ray"),
            FormatOption::new("laserdisc", "LaserDisc"),
            FormatOption::new("streaming", "Streaming"),
            FormatOption::new("broadcast", "Broadcast"),
            FormatOption::new("telecine", "Telecine"),
        ],
        flag_labels: vec![
            FormatOption::new("three_d", "3D"),
            FormatOption::new("hfr", "HFR"),
            FormatOption::new("dbox", "D-BOX"),
        ],
        legal,
    }
}

/// The display label of a technology.
fn tech_label(projection: &Projection) -> String {
    match projection {
        Projection::Dci(dci) => match dci {
            Dci::Lamp2K => "2K xenon",
            Dci::Laser2K => "2K laser",
            Dci::Lamp4K => "4K xenon",
            Dci::Laser4K => "4K laser",
            Dci::Custom(id) => id,
        },
        Projection::Imax(imax) => match imax {
            Imax::Xenon2K => "2K xenon",
            Imax::Laser4KSingle => "4K single laser",
            Imax::Laser4KDual => "4K dual laser",
            Imax::Film70mm => "70mm",
            Imax::Custom(id) => id,
        },
        Projection::Film(film) => match film {
            Film::Mm35 => "35mm",
            Film::Mm70 => "70mm",
            Film::Custom(id) => id,
        },
    }
    .to_owned()
}

/// The display label of an auditorium class.
fn screen_label(screen: &ScreenClass) -> String {
    match screen {
        ScreenClass::Regular => "Regular",
        ScreenClass::Plf => "PLF",
        ScreenClass::Vip => "VIP",
        ScreenClass::FourDx => "4DX",
        ScreenClass::ScreenX => "ScreenX",
        ScreenClass::ImaxClassic => "IMAX 1.43:1",
        ScreenClass::ImaxRetrofit => "IMAX 1.90:1",
        ScreenClass::Ultra4Dx => "Ultra 4DX",
        ScreenClass::Custom(id) => id,
    }
    .to_owned()
}

/// The display label of a home release medium.
fn medium_label(medium: &HomeMedium) -> String {
    match medium {
        HomeMedium::Dvd => "DVD",
        HomeMedium::HdDvd => "HD DVD",
        HomeMedium::BluRay => "Blu-ray",
        HomeMedium::UhdBluRay => "4K Ultra HD Blu-ray",
        HomeMedium::LaserDisc => "LaserDisc",
        HomeMedium::Streaming => "Streaming",
        HomeMedium::Broadcast => "Broadcast",
        HomeMedium::Telecine => "Telecine",
        HomeMedium::Custom(id) => id,
    }
    .to_owned()
}

impl Format {
    /// The one-line display summary of the format, e.g.
    /// `"4K laser · PLF · 3D"` or `"Blu-ray"`.
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Format::Theatre {
                projection,
                screen,
                three_d,
                hfr,
                dbox,
            } => {
                let mut parts = vec![tech_label(projection), screen_label(screen)];
                for (flag, on) in [("three_d", *three_d), ("hfr", *hfr), ("dbox", *dbox)] {
                    if on {
                        parts.push(flag_label(flag).to_owned());
                    }
                }
                parts.join(" · ")
            }
            Format::Home { medium } => medium_label(medium),
        }
    }
}

/// The display label of a presentation flag key.
fn flag_label(key: &str) -> &'static str {
    match key {
        "three_d" => "3D",
        "hfr" => "HFR",
        _ => "D-BOX",
    }
}

#[cfg(test)]
mod tests {
    use super::format_catalog;
    use crate::domain::{Dci, Film, Flag, Format, HomeMedium, Imax, Projection, ScreenClass};
    use crate::error::Error;

    /// Build the concrete projection for `(family, technology)`.
    fn projection(family: &str, technology: &str) -> Projection {
        match (family, technology) {
            ("dci", "2k_lamp") => Projection::Dci(Dci::Lamp2K),
            ("dci", "2k_laser") => Projection::Dci(Dci::Laser2K),
            ("dci", "4k_lamp") => Projection::Dci(Dci::Lamp4K),
            ("dci", _) => Projection::Dci(Dci::Laser4K),
            ("imax", "2k_xenon") => Projection::Imax(Imax::Xenon2K),
            ("imax", "4k_single_laser") => Projection::Imax(Imax::Laser4KSingle),
            ("imax", "4k_dual_laser") => Projection::Imax(Imax::Laser4KDual),
            ("imax", _) => Projection::Imax(Imax::Film70mm),
            ("film", "35mm") => Projection::Film(Film::Mm35),
            ("film", _) => Projection::Film(Film::Mm70),
            (_, _) => unreachable!(),
        }
    }

    /// Build the concrete auditorium for a key.
    fn screen(key: &str) -> ScreenClass {
        match key {
            "regular" => ScreenClass::Regular,
            "plf" => ScreenClass::Plf,
            "vip" => ScreenClass::Vip,
            "4dx" => ScreenClass::FourDx,
            "screenx" => ScreenClass::ScreenX,
            "imax_class" => ScreenClass::ImaxClassic,
            "imax_retrofit" => ScreenClass::ImaxRetrofit,
            "ultra_4dx" => ScreenClass::Ultra4Dx,
            _ => unreachable!(),
        }
    }

    fn format_for(
        family: &str,
        technology: &str,
        screen_key: &str,
        three_d: bool,
        hfr: bool,
        dbox: bool,
    ) -> Format {
        Format::Theatre {
            projection: projection(family, technology),
            screen: screen(screen_key),
            three_d,
            hfr,
            dbox,
        }
    }

    #[test]
    fn every_legal_row_validates() {
        for row in format_catalog().legal {
            let format = format_for(
                &row.family,
                &row.technology,
                &row.screen,
                row.forced_flags.iter().any(|f| f == "three_d"),
                row.forced_flags.iter().any(|f| f == "hfr"),
                row.forced_flags.iter().any(|f| f == "dbox"),
            );
            assert!(
                format.validate().is_ok(),
                "legal row should validate: {row:?} -> {format:?}"
            );
        }
    }

    #[test]
    fn every_illegal_pair_is_rejected() {
        let catalog = format_catalog();
        for family in &catalog.families {
            for technology in &catalog.technologies[&family.key] {
                for candidate in [
                    "regular",
                    "plf",
                    "vip",
                    "4dx",
                    "screenx",
                    "imax_class",
                    "imax_retrofit",
                    "ultra_4dx",
                ] {
                    let legal = catalog.legal.iter().any(|row| {
                        row.family == family.key
                            && row.technology == technology.key
                            && row.screen == candidate
                    });
                    if legal {
                        continue;
                    }
                    let format =
                        format_for(&family.key, &technology.key, candidate, false, false, false);
                    assert!(
                        matches!(
                            format.validate(),
                            Err(Error::ProjectionScreenMismatch { .. })
                        ),
                        "illegal pair should be rejected: {family:?} {technology:?} {candidate}"
                    );
                }
            }
        }
    }

    #[test]
    fn disallowed_and_forced_flags_are_rejected() {
        let catalog = format_catalog();
        for row in &catalog.legal {
            for (key, flag, index) in [
                ("three_d", Flag::ThreeD, 0),
                ("hfr", Flag::Hfr, 1),
                ("dbox", Flag::Dbox, 2),
            ] {
                let forced = row.forced_flags.iter().any(|f| f == key);
                let allowed = row.allowed_flags.iter().any(|f| f == key);
                if !allowed {
                    let mut on = [
                        row.forced_flags.iter().any(|f| f == "three_d"),
                        row.forced_flags.iter().any(|f| f == "hfr"),
                        row.forced_flags.iter().any(|f| f == "dbox"),
                    ];
                    on[index] = true;
                    let format = format_for(
                        &row.family,
                        &row.technology,
                        &row.screen,
                        on[0],
                        on[1],
                        on[2],
                    );
                    assert!(
                        matches!(format.validate(), Err(Error::FlagMismatch { flag: bad, .. }) if bad == flag),
                        "disallowed flag should be rejected: {row:?} {key}"
                    );
                }
                if forced {
                    // Leaving the forced flag off is rejected.
                    let format = format_for(
                        &row.family,
                        &row.technology,
                        &row.screen,
                        false,
                        false,
                        false,
                    );
                    assert!(
                        matches!(format.validate(), Err(Error::FlagMismatch { flag: bad, .. }) if bad == flag),
                        "missing forced flag should be rejected: {row:?} {key}"
                    );
                }
            }
        }
    }

    #[test]
    fn summary_matches_labels() {
        assert_eq!(
            Format::Home {
                medium: HomeMedium::BluRay
            }
            .summary(),
            "Blu-ray"
        );
        assert_eq!(
            format_for("dci", "4k_laser", "plf", true, false, false).summary(),
            "4K laser · PLF · 3D"
        );
        assert_eq!(
            format_for("imax", "4k_dual_laser", "imax_class", true, true, false).summary(),
            "4K dual laser · IMAX 1.43:1 · 3D · HFR"
        );
        assert_eq!(
            Format::Home {
                medium: HomeMedium::Custom("VHS".into())
            }
            .summary(),
            "VHS"
        );
    }

    #[test]
    fn imax_offers_only_imax_auditoria() {
        let catalog = format_catalog();
        let imax = &catalog.screens["imax"];
        assert_eq!(imax.len(), 2);
        assert!(imax.iter().all(|s| s.key.starts_with("imax_")));
        assert_eq!(catalog.screens["film"].len(), 1);
    }
}
