// KMRP Font Converter: writing a set of game fonts into a folder, all or nothing.
//
// Every font is made in a folder of its own inside the output folder first; only when all
// of them are made, and every file they replace can be replaced, are they moved into place,
// one rename each (audit F1 to F3, 2026-11-08). A failure half way leaves the folder as it
// was.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use crate::atlas::{self, Reference, Target};
use crate::font::LoadedFace;

pub const TPC_BACKUP: &str = ".before-font-converter";
const EXTS: [&str; 3] = [".tga", ".txi", ".tpc"];

pub struct Conversion {
    pub folder: PathBuf,
    pub target: Target,
    pub references: Arc<HashMap<String, Reference>>,
    /// Each game font name and the face it is made from.
    pub work: Vec<(String, Arc<LoadedFace>)>,
    /// How many fonts are made.
    pub progress: Arc<AtomicUsize>,
    /// Set by the window: stops the conversion with nothing written.
    pub stop: Arc<AtomicBool>,
    /// Set once files start to move into the folder: a failure after it may have left some
    /// fonts new and some old there.
    pub moving: bool,
}

pub enum Failure {
    Stopped,
    Failed(String),
}

/// How many font files the folder already has under the names these fonts use: what Convert
/// asks about before replacing them.
pub fn existing<'a>(folder: &Path, names: impl IntoIterator<Item = &'a str>) -> usize {
    names
        .into_iter()
        .map(|n| EXTS.iter().filter(|ext| folder.join(format!("{n}{ext}")).is_file()).count())
        .sum()
}

impl Conversion {
    /// Makes every font, then puts them in place. Err with a message for the player when it
    /// cannot; then nothing in the folder has changed, unless the moves themselves fail.
    pub fn run(&mut self, notify: &dyn Fn()) -> Result<(), Failure> {
        let fail = |e: std::io::Error| Failure::Failed(e.to_string());
        std::fs::create_dir_all(&self.folder).map_err(fail)?;
        let work = self.folder.join(format!(".kmrp-font-converter-{:08x}", random()));
        std::fs::create_dir_all(&work).map_err(fail)?;
        let result = self.make_and_move(&work, notify);
        let _ = std::fs::remove_dir_all(&work);
        result
    }

    fn make_and_move(&mut self, work: &Path, notify: &dyn Fn()) -> Result<(), Failure> {
        let stopped = || self.stop.load(Ordering::SeqCst);
        for (done, (name, face)) in self.work.iter().enumerate() {
            if stopped() {
                return Err(Failure::Stopped);
            }
            let reference = self
                .references
                .get(name)
                .ok_or_else(|| Failure::Failed(format!("No measurements for {name}.")))?;
            atlas::write(face, reference, work, self.target).map_err(|e| Failure::Failed(e.0))?;
            self.progress.store(done + 1, Ordering::SeqCst);
            notify();
        }
        if stopped() {
            return Err(Failure::Stopped);
        }
        let mut names: Vec<&str> = Vec::new();
        for (n, _) in &self.work {
            if !names.contains(&n.as_str()) {
                names.push(n);
            }
        }
        // Everything that will be replaced must be replaceable before anything moves.
        for name in &names {
            for ext in EXTS {
                let path = self.folder.join(format!("{name}{ext}"));
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.is_file() && meta.permissions().readonly() {
                        return Err(Failure::Failed(format!(
                            "{name}{ext} in the folder is read-only, so it can't be replaced. Nothing was changed."
                        )));
                    }
                }
            }
        }
        self.moving = true;
        let fail = |e: std::io::Error| Failure::Failed(e.to_string());
        for name in &names {
            // A .tpc of the same name would win over the new .tga: set aside, under a name no
            // earlier conversion took.
            let tpc = self.folder.join(format!("{name}.tpc"));
            if tpc.is_file() {
                let mut aside = tpc.as_os_str().to_owned();
                aside.push(TPC_BACKUP);
                std::fs::rename(&tpc, free(PathBuf::from(aside))).map_err(fail)?;
            }
            for ext in [".tga", ".txi"] {
                let target = self.folder.join(format!("{name}{ext}"));
                if target.is_file() {
                    std::fs::remove_file(&target).map_err(fail)?;
                }
                std::fs::rename(work.join(format!("{name}{ext}")), &target).map_err(fail)?;
            }
        }
        Ok(())
    }
}

// `path`, or path-2, path-3... when it is taken (audit F2: a second conversion stopped at a
// .tpc already set aside).
fn free(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    for n in 2.. {
        let mut p = path.as_os_str().to_owned();
        p.push(format!("-{n}"));
        let p = PathBuf::from(p);
        if !p.exists() {
            return p;
        }
    }
    unreachable!()
}

fn random() -> u32 {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0));
    h.write_u32(std::process::id());
    h.finish() as u32
}
