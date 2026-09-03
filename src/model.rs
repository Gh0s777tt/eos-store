//! The headless half of E-OS Store — everything the GUI shows but nothing that
//! needs a display.
//!
//! The skeleton ships a deliberately small document catalogue rather than an
//! empty module: it gives the CI gates something that can actually fail, and it
//! fixes the shape every eos-store feature will grow into (identified entries,
//! a case-insensitive filter, a status line the window renders).

/// What the model refuses to do, and why.
#[derive(Debug, PartialEq, Eq)]
pub enum ModelError {
    /// A title made only of whitespace carries no information; the UI would
    /// render an unclickable empty row.
    BlankTitle,
    /// The identifier counter reached `u32::MAX`. Handing out a wrapped id
    /// would alias two entries — with `overflow-checks = true` the wrap would
    /// abort the process instead, so the model refuses first and says why.
    IdSpaceExhausted,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::BlankTitle => write!(f, "entry title is empty or blank"),
            ModelError::IdSpaceExhausted => write!(f, "entry id space exhausted"),
        }
    }
}

impl std::error::Error for ModelError {}

/// One catalogue row: a stable identifier plus the label the window shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Stable identifier, never reused after a removal.
    pub id: u32,
    /// Display label, trimmed of surrounding whitespace.
    pub title: String,
}

/// The in-memory document catalogue of E-OS Store.
///
/// Persistence is deliberately absent — the skeleton stops at the boundary
/// where each product's storage decision starts (a spreadsheet file, a slide
/// deck, a remote drive, a package index).
#[derive(Debug, Default)]
pub struct Catalog {
    entries: Vec<Entry>,
    next_id: u32,
}

impl Catalog {
    /// An empty catalogue whose first handed-out identifier is 1.
    pub fn new() -> Self {
        Catalog {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    /// Number of entries currently held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the catalogue holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Append an entry and return its identifier.
    ///
    /// Fails on a blank title and refuses to wrap the identifier counter.
    pub fn add(&mut self, title: &str) -> Result<u32, ModelError> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(ModelError::BlankTitle);
        }
        let id = self.next_id;
        // Checked, not wrapping: see ModelError::IdSpaceExhausted.
        self.next_id = id.checked_add(1).ok_or(ModelError::IdSpaceExhausted)?;
        self.entries.push(Entry {
            id,
            title: trimmed.to_string(),
        });
        Ok(id)
    }

    /// Remove the entry with `id`; returns whether anything was removed.
    pub fn remove(&mut self, id: u32) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() != before
    }

    /// Entries whose title contains `query`, compared case-insensitively.
    ///
    /// An empty (or whitespace-only) query matches everything — that is the
    /// "no filter typed yet" state of the search box, not a match failure.
    pub fn find(&self, query: &str) -> Vec<&Entry> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return self.entries.iter().collect();
        }
        self.entries
            .iter()
            .filter(|e| e.title.to_lowercase().contains(&needle))
            .collect()
    }

    /// The one-line status the window shows under the content area.
    pub fn status_line(&self) -> String {
        format!(
            "{} {} — status: skeleton · {} entries",
            APP_TITLE,
            APP_VERSION,
            self.len()
        )
    }
}

/// Product display title, rendered in the window title bar and the header.
pub const APP_TITLE: &str = "E-OS Store";
/// Crate name, kept in sync with Cargo.toml by the compiler, not by hand.
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
/// Crate version, kept in sync with Cargo.toml by the compiler, not by hand.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Marker printed by `--selftest`; boot probes and CI grep for this exact text.
pub const SELFTEST_OK: &str = "EOS-STORE-SELFTEST-OK";

/// Headless proof that the model works: exercised by `eos-store --selftest`
/// on the host and inside the E-OS image, where no display exists.
///
/// Returns the failing invariant instead of panicking, so the caller can print
/// it next to the FAIL marker.
pub fn selftest() -> Result<(), String> {
    let mut cat = Catalog::new();
    if !cat.is_empty() {
        return Err("a fresh catalogue was not empty".to_string());
    }
    let a = cat.add("Alpha report").map_err(|e| e.to_string())?;
    let b = cat.add("  beta draft  ").map_err(|e| e.to_string())?;
    if a == b {
        return Err(format!("identifiers collided: {a} == {b}"));
    }
    if cat.len() != 2 {
        return Err(format!("expected 2 entries, got {}", cat.len()));
    }
    if cat.find("BETA").len() != 1 {
        return Err("case-insensitive search did not find 'beta draft'".to_string());
    }
    if !cat.find("nothing-matches-this").is_empty() {
        return Err("search matched an absent entry".to_string());
    }
    if cat.add("   ").is_ok() {
        return Err("blank title was accepted".to_string());
    }
    if !cat.remove(a) {
        return Err(format!("entry {a} could not be removed"));
    }
    if cat.remove(a) {
        return Err(format!("entry {a} was removed twice"));
    }
    if !cat.status_line().contains(APP_VERSION) {
        return Err("status line lost the version".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Contract (level 1) plus failure path (level 3): a title that carries no
    /// characters is refused, and the refusal is named.
    #[test]
    fn add_refuses_blank_titles() {
        let mut cat = Catalog::new();
        assert_eq!(cat.add(""), Err(ModelError::BlankTitle));
        assert_eq!(cat.add("   \t\n "), Err(ModelError::BlankTitle));
        assert!(cat.is_empty());
        assert!(cat.add(" Quarterly plan ").is_ok());
        // The stored title is trimmed, not the raw input.
        assert_eq!(cat.find("").len(), 1);
        assert_eq!(cat.find("")[0].title, "Quarterly plan");
    }

    /// The filter must both match and *not* match — a search that returns
    /// everything would pass a match-only assertion.
    #[test]
    fn find_matches_case_insensitively_and_narrows() {
        let mut cat = Catalog::new();
        cat.add("Budget 2026").unwrap();
        cat.add("budget archive").unwrap();
        cat.add("Roadmap").unwrap();
        assert_eq!(cat.find("BUDGET").len(), 2);
        assert_eq!(cat.find("roadmap").len(), 1);
        assert_eq!(cat.find("ledger").len(), 0);
        // An empty query is "no filter", not "no match".
        assert_eq!(cat.find("   ").len(), 3);
    }

    /// Identifiers are unique and are never reused after a removal — reuse
    /// would silently re-point an open editor at a different document.
    #[test]
    fn identifiers_are_unique_and_not_reused() {
        let mut cat = Catalog::new();
        let a = cat.add("one").unwrap();
        let b = cat.add("two").unwrap();
        assert_ne!(a, b);
        assert!(cat.remove(a));
        let c = cat.add("three").unwrap();
        assert_ne!(c, a);
        assert_ne!(c, b);
        assert_eq!(cat.len(), 2);
    }

    /// The counter refuses to wrap. With `overflow-checks = true` a wrap would
    /// abort the release binary; the model turns it into a named error.
    #[test]
    fn identifier_counter_refuses_to_wrap() {
        let mut cat = Catalog::new();
        cat.next_id = u32::MAX;
        assert_eq!(cat.add("last"), Err(ModelError::IdSpaceExhausted));
        // The refused entry was not stored.
        assert!(cat.is_empty());
    }

    /// Anti-drift: the status line is built from the manifest, so a version
    /// bump cannot leave a stale string in the window (the SYNC-marker lesson
    /// from CLAUDE.md, applied to a string this crate owns).
    #[test]
    fn status_line_carries_title_and_manifest_version() {
        let cat = Catalog::new();
        let line = cat.status_line();
        assert!(
            line.contains(APP_TITLE),
            "status line lost the title: {line}"
        );
        assert!(
            line.contains(env!("CARGO_PKG_VERSION")),
            "status line lost the version: {line}"
        );
        assert_eq!(APP_NAME, env!("CARGO_PKG_NAME"));
        assert!(SELFTEST_OK.ends_with("-SELFTEST-OK"));
    }

    /// The headless proof the boot probe runs must itself pass here, so a
    /// broken `--selftest` is caught by `cargo test` and not only in QEMU.
    #[test]
    fn selftest_passes_on_the_host() {
        assert_eq!(selftest(), Ok(()));
    }
}
