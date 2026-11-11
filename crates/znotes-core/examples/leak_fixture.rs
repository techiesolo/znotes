use std::path::Path;

use znotes_core::repo::notes;
use znotes_core::{KdfParams, Vault};

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(dir), Some(marker)) = (args.next(), args.next()) else {
        eprintln!("usage: leak_fixture <vault_dir> <marker>   (password is `pw`)");
        std::process::exit(2);
    };
    let vault =
        Vault::create(Path::new(&dir), "pw", KdfParams::fast_for_tests()).expect("create vault");
    let note = notes::create(vault.conn(), None, &format!("Title {marker}")).expect("create note");
    notes::update(
        vault.conn(),
        note.id,
        &format!("Title {marker}"),
        &format!("API_KEY=sk-{marker}\n\nbody mentions {marker} twice: {marker}"),
    )
    .expect("update note");
    println!("vault with marker written to {dir}; unlock with password `pw`");
}
