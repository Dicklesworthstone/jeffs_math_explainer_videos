//! The preamble-pack registry: `tex_templates.yml` reborn as named fmd-math
//! preamble packs (§11.4, §13.6).
//!
//! The Reference's `tex.template` key named a LaTeX preamble + compiler
//! pair; FrankenManim's typesetting is native, so the concept becomes a
//! **preamble pack** — a named macro/symbol bundle fmd-math loads before
//! laying out a formula. This module owns the registry *surface*: pack
//! naming, lookup, and the compatibility mapping from the common Reference
//! template names. Pack **content** (the actual `\newcommand`-tier macro
//! bundles and symbol sets) is W6's business and lands with fm-kg9 — the
//! [`Pack::content_id`] is the stable handle W6 keys its bundles on.
//!
//! # The compatibility mapping
//!
//! `tex_templates.yml` ships two families: the general-purpose templates
//! (`default`, `basic`, `empty`, and their `*_ctex` CJK variants) and a
//! showcase of ~30 host-font templates (`american_typewriter`, …). The
//! mapping keeps what has native meaning and names what does not:
//!
//! - `default` / `basic` / `empty` → the same-named native packs.
//! - The `*_ctex` variants → a **named capability error**: CJK typesetting
//!   is outside the current typography tier (the Behavior Notes' honest
//!   fringe), with the revisit trigger documented there.
//! - The host-font showcase templates → a named error too: they configure
//!   external LaTeX around system fonts, neither of which exists here.
//! - Anything else → an unknown-template error listing the known packs.
//!
//! Every refusal is precise and actionable — never a silent fallback to a
//! different look (D5: no silent substitution, ever).

use core::fmt;

/// A named preamble pack: the registry surface W6's content lands behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pack {
    /// The pack name — the value of the `tex.template` config key.
    pub name: &'static str,
    /// One-line description for `fmn doctor` and error listings.
    pub description: &'static str,
    /// The stable identifier fmd-math's macro/symbol bundles key on
    /// (fm-kg9 consumes this; the id is versioned there, not here).
    pub content_id: &'static str,
}

/// Why a template name did not resolve to a pack. Every variant carries the
/// full story: what was asked, why it cannot be served, what exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackError {
    /// A Reference template whose meaning cannot be reproduced natively,
    /// with the specific reason (CJK tier, host fonts, …).
    UnsupportedTemplate {
        /// The requested template name.
        template: String,
        /// Why it has no native pack.
        reason: &'static str,
        /// The packs that do exist.
        known: Vec<&'static str>,
    },
    /// A name the registry has never heard of.
    UnknownTemplate {
        /// The requested template name.
        template: String,
        /// The packs that do exist.
        known: Vec<&'static str>,
    },
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTemplate {
                template,
                reason,
                known,
            } => write!(
                f,
                "tex template {template:?} has no native preamble pack: {reason}; available packs: {}",
                known.join(", ")
            ),
            Self::UnknownTemplate { template, known } => write!(
                f,
                "unknown tex template {template:?}; available packs: {}",
                known.join(", ")
            ),
        }
    }
}

impl std::error::Error for PackError {}

/// The built-in packs. `default` is the Reference's everyday surface
/// (amsmath/amssymb-tier macros and symbols); `basic` is its slimmer
/// sibling; `empty` is no preamble at all.
const BUILTIN: &[Pack] = &[
    Pack {
        name: "default",
        description: "the standard macro and symbol bundle (amsmath/amssymb-tier)",
        content_id: "fmd-math/pack/default",
    },
    Pack {
        name: "basic",
        description: "a minimal macro and symbol bundle",
        content_id: "fmd-math/pack/basic",
    },
    Pack {
        name: "empty",
        description: "no preamble: bare TeX-math primitives only",
        content_id: "fmd-math/pack/empty",
    },
];

const CJK_REASON: &str = "it selects CJK typesetting, which is outside the current typography \
     tier (complex scripts are tiered out with a revisit trigger; see the Behavior Notes)";
const HOST_FONT_REASON: &str = "it configures external LaTeX around a host system font; \
     FrankenManim typesets natively on bundled faces";

/// The Reference templates that exist but cannot be served natively, with
/// their reasons. (The host-font showcase list is every remaining name in
/// the shipped `tex_templates.yml`.)
const UNSUPPORTED: &[(&str, &str)] = &[
    ("ctex", CJK_REASON),
    ("basic_ctex", CJK_REASON),
    ("empty_ctex", CJK_REASON),
    ("american_typewriter", HOST_FONT_REASON),
    ("antykwa", HOST_FONT_REASON),
    ("apple_chancery", HOST_FONT_REASON),
    ("auriocus_kalligraphicus", HOST_FONT_REASON),
    ("baskervald_adf_fourier", HOST_FONT_REASON),
    ("baskerville_it", HOST_FONT_REASON),
    ("biolinum", HOST_FONT_REASON),
    ("brushscriptx", HOST_FONT_REASON),
    ("chalkboard_se", HOST_FONT_REASON),
    ("chalkduster", HOST_FONT_REASON),
    ("comfortaa", HOST_FONT_REASON),
    ("comic_sans", HOST_FONT_REASON),
    ("droid_sans", HOST_FONT_REASON),
    ("droid_sans_it", HOST_FONT_REASON),
    ("droid_serif", HOST_FONT_REASON),
    ("droid_serif_px_it", HOST_FONT_REASON),
    ("ecf_augie", HOST_FONT_REASON),
    ("ecf_jd", HOST_FONT_REASON),
    ("ecf_skeetch", HOST_FONT_REASON),
    ("ecf_tall_paul", HOST_FONT_REASON),
    ("ecf_webster", HOST_FONT_REASON),
    ("electrum_adf", HOST_FONT_REASON),
    ("epigrafica", HOST_FONT_REASON),
    ("fourier_utopia", HOST_FONT_REASON),
    ("french_cursive", HOST_FONT_REASON),
    ("gfs_bodoni", HOST_FONT_REASON),
    ("gfs_didot", HOST_FONT_REASON),
    ("gfs_neohellenic", HOST_FONT_REASON),
    ("gnu_freesans_tx", HOST_FONT_REASON),
    ("gnu_freeserif_freesans", HOST_FONT_REASON),
    ("helvetica_fourier_it", HOST_FONT_REASON),
    ("latin_modern_tw", HOST_FONT_REASON),
    ("latin_modern_tw_it", HOST_FONT_REASON),
    ("libertine", HOST_FONT_REASON),
    ("libris_adf_fourier", HOST_FONT_REASON),
    ("minion_pro_myriad_pro", HOST_FONT_REASON),
    ("minion_pro_tx", HOST_FONT_REASON),
    ("new_century_schoolbook", HOST_FONT_REASON),
    ("new_century_schoolbook_px", HOST_FONT_REASON),
    ("noteworthy_light", HOST_FONT_REASON),
    ("palatino", HOST_FONT_REASON),
    ("papyrus", HOST_FONT_REASON),
    ("romande_adf_fourier_it", HOST_FONT_REASON),
    ("slitex", HOST_FONT_REASON),
    ("times_fourier_it", HOST_FONT_REASON),
    ("urw_avant_garde", HOST_FONT_REASON),
    ("urw_zapf_chancery", HOST_FONT_REASON),
    ("venturis_adf_fourier_it", HOST_FONT_REASON),
    ("verdana_it", HOST_FONT_REASON),
    ("vollkorn", HOST_FONT_REASON),
    ("vollkorn_fourier_it", HOST_FONT_REASON),
    ("zapf_chancery", HOST_FONT_REASON),
];

/// The preamble-pack registry: builtin packs plus the Reference-template
/// compatibility mapping.
#[derive(Clone, Debug, Default)]
pub struct PackRegistry {}

impl PackRegistry {
    /// The registry of builtin packs.
    #[must_use]
    pub fn builtin() -> Self {
        Self {}
    }

    /// The packs, in a stable order.
    #[must_use]
    pub fn packs(&self) -> &'static [Pack] {
        BUILTIN
    }

    /// The pack names, for listings and error messages.
    #[must_use]
    pub fn names(&self) -> Vec<&'static str> {
        BUILTIN.iter().map(|p| p.name).collect()
    }

    /// A pack by its own name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&'static Pack> {
        BUILTIN.iter().find(|p| p.name == name)
    }

    /// Resolve a `tex.template` config value through the compatibility
    /// mapping: a native pack, or a precise refusal.
    ///
    /// # Errors
    /// [`PackError::UnsupportedTemplate`] for a known Reference template
    /// with no native meaning; [`PackError::UnknownTemplate`] otherwise.
    pub fn resolve_template(&self, template: &str) -> Result<&'static Pack, PackError> {
        if let Some(pack) = self.get(template) {
            return Ok(pack);
        }
        if let Some((_, reason)) = UNSUPPORTED.iter().find(|(name, _)| *name == template) {
            return Err(PackError::UnsupportedTemplate {
                template: template.to_owned(),
                reason,
                known: self.names(),
            });
        }
        Err(PackError::UnknownTemplate {
            template: template.to_owned(),
            known: self.names(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_common_templates_resolve_to_native_packs() {
        let r = PackRegistry::builtin();
        for name in ["default", "basic", "empty"] {
            let pack = r.resolve_template(name).expect(name);
            assert_eq!(pack.name, name);
            assert!(pack.content_id.starts_with("fmd-math/pack/"));
        }
    }

    #[test]
    fn cjk_templates_refuse_with_the_tier_reason() {
        let r = PackRegistry::builtin();
        for name in ["ctex", "basic_ctex", "empty_ctex"] {
            match r.resolve_template(name) {
                Err(PackError::UnsupportedTemplate {
                    template,
                    reason,
                    known,
                }) => {
                    assert_eq!(template, name);
                    assert!(reason.contains("CJK"), "{reason}");
                    assert_eq!(known, vec!["default", "basic", "empty"]);
                }
                other => panic!("{name}: expected UnsupportedTemplate, got {other:?}"),
            }
        }
    }

    #[test]
    fn host_font_templates_refuse_with_the_native_reason() {
        let r = PackRegistry::builtin();
        match r.resolve_template("comic_sans") {
            Err(PackError::UnsupportedTemplate { reason, .. }) => {
                assert!(reason.contains("host system font"), "{reason}");
            }
            other => panic!("expected UnsupportedTemplate, got {other:?}"),
        }
    }

    #[test]
    fn unknown_templates_list_what_exists() {
        let r = PackRegistry::builtin();
        match r.resolve_template("my_custom_thing") {
            Err(e @ PackError::UnknownTemplate { .. }) => {
                let msg = e.to_string();
                assert!(msg.contains("my_custom_thing"), "{msg}");
                assert!(msg.contains("default, basic, empty"), "{msg}");
            }
            other => panic!("expected UnknownTemplate, got {other:?}"),
        }
    }

    #[test]
    fn refusals_are_never_silent_fallbacks() {
        // The registry must never map an unsupported template onto a pack
        // with a different look (D5).
        let r = PackRegistry::builtin();
        assert!(r.resolve_template("ctex").is_err());
        assert!(r.get("ctex").is_none());
    }
}
