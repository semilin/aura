//! A layer over the `alpm` library to aid with common tasks.

use alpm::{Alpm, AlpmList, Db, IntoIter, Package, PackageReason};

/// The default filepath of the Pacman configuration.
pub const DEFAULT_PAC_CONF: &str = "/etc/pacman.conf";

// TODO Handle the other potential default locations.
/// The default filepath of the Makepkg configuration.
pub const DEFAULT_MAKEPKG_CONF: &str = "/etc/makepkg.conf";

/// An `Iterator` that yields installed packages which were installed from the
/// AUR, or are otherwise not from official repositories.
pub struct Foreigns<'a> {
    /// Local package iterator.
    locals: IntoIter<'a, Package<'a>>,
    /// Sync DBs.
    syncs: AlpmList<'a, Db<'a>>,
}

impl<'a> Iterator for Foreigns<'a> {
    type Item = Package<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.locals.next().and_then(|p| {
            let name = p.name();
            match self.syncs.iter().filter_map(|db| db.pkg(name).ok()).next() {
                None => Some(p),
                Some(_) => self.next(),
            }
        })
    }
}

// TODO Make this an iterator.
/// All orphaned packages.
///
/// An orphan is a package that was installed as a dependency, but whose parent
/// package is no longer installed.
pub fn orphans(alpm: &Alpm) -> Vec<Package> {
    alpm.localdb()
        .pkgs()
        .iter()
        .filter(|p| {
            p.reason() == PackageReason::Depend
                && p.required_by().is_empty()
                && p.optional_for().is_empty()
        })
        .collect()
}

/// All foreign packages as an `Iterator`.
pub fn foreigns(alpm: &Alpm) -> Foreigns {
    let locals = alpm.localdb().pkgs().into_iter();
    let syncs = alpm.syncdbs();
    Foreigns { locals, syncs }
}
