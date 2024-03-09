use std::fs::File;

use hemtt_pbo::ReadablePbo;
use hemtt_signing::BIPrivateKey;
use indicatif::ProgressBar;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn main() {
    let mut addons = Vec::new();
    for dirs in std::fs::read_dir(".").expect("can't read root dir") {
        let dir = dirs.expect("can't read dir");
        if !dir.path().is_dir() {
            continue;
        }
        if !dir
            .file_name()
            .to_str()
            .expect("can't convert dir name to string")
            .starts_with('@')
        {
            continue;
        }
        let keys = dir.path().join("keys");
        if keys.exists() {
            std::fs::remove_dir_all(keys).expect("can't remove keys dir");
        }
        for addon in std::fs::read_dir(dir.path().join("addons")).expect("can't read addons dir") {
            let addon = addon.expect("can't read addon");
            if addon.path().extension() == Some(std::ffi::OsStr::new("bisign")) {
                std::fs::remove_file(addon.path()).expect("can't remove bikey");
            }
            if addon.path().extension() == Some(std::ffi::OsStr::new("pbo")) {
                addons.push(addon.path());
            }
        }
    }
    println!("Signing {} addons", addons.len());

    let private =
        BIPrivateKey::generate(2048, "synixe_resign").expect("can't generate private key");
    let public = private.to_public_key();

    let pb = ProgressBar::new(addons.len() as u64);
    addons.par_iter().for_each(|addon| {
        let result = std::panic::catch_unwind(|| {
            let sig = private
                .sign(
                    &mut ReadablePbo::from(File::open(addon).expect("can't open pbo"))
                        .expect("can't read pbo"),
                    hemtt_pbo::BISignVersion::V3,
                )
                .expect("can't sign pbo");
            let addon_sig = addon.with_extension("synixe_resign.bisign");
            sig.write(&mut File::create(addon_sig).expect("can't create bikey"))
                .expect("can't write bikey");
        });
        if result.is_err() {
            println!("Failed to sign {}", addon.display());
            eprintln!("{:?}", result);
        }
        pb.inc(1);
    });

    let _ = std::fs::remove_dir_all("synixe_resign");
    std::fs::create_dir("synixe_resign").expect("can't create resign mod");
    public
        .write(&mut File::create("synixe_resign/synixe_resign.bikey").expect("can't create bikey"))
        .expect("can't write bikey");

    pb.finish_with_message("Done, created synixe_resign.bikey");
}
