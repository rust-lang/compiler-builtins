use object::read::archive::{ArchiveFile, ArchiveMember};
use object::{Object, ObjectSymbol, Symbol, SymbolKind, SymbolScope, SymbolSection};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const USAGE: &str = "Usage:

    symbol-check check-duplicates PATHS...
    symbol-check check-core-syms PATHS...
";

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let args_ref = args.iter().map(String::as_str).collect::<Vec<_>>();

    match &args_ref[1..] {
        ["check-duplicates", rest @ ..] if !rest.is_empty() => {
            rest.iter().for_each(verify_no_duplicates);
        }
        ["check-core-syms", rest @ ..] if !rest.is_empty() => {
            rest.iter().for_each(verify_core_symbols);
        }
        _ => {
            println!("{USAGE}");
            std::process::exit(1);
        }
    }
}

#[expect(unused)] // only for printing
#[derive(Clone, Debug)]
struct SymInfo {
    name: String,
    kind: SymbolKind,
    scope: SymbolScope,
    section: SymbolSection,
    is_undefined: bool,
    is_global: bool,
    is_local: bool,
    is_weak: bool,
    is_common: bool,
    address: u64,
    object: String,
}

impl SymInfo {
    fn new(sym: &Symbol, member: &ArchiveMember) -> Self {
        Self {
            name: sym.name().expect("missing name").to_owned(),
            kind: sym.kind(),
            scope: sym.scope(),
            section: sym.section(),
            is_undefined: sym.is_undefined(),
            is_global: sym.is_global(),
            is_local: sym.is_local(),
            is_weak: sym.is_weak(),
            is_common: sym.is_common(),
            address: sym.address(),
            object: String::from_utf8_lossy(member.name()).into_owned(),
        }
    }
}

/// Ensure that the same global symbol isn't defined in multiple object files within an archive.
fn verify_no_duplicates(path: impl AsRef<Path>) {
    println!("Checking for duplicates at {}", path.as_ref().display());

    let mut syms = BTreeMap::<String, SymInfo>::new();
    let mut dups = Vec::new();

    for_each_symbol(path, |sym, member| {
        if !sym.is_global() || sym.is_undefined() {
            // Only check defined globals
            return;
        }

        let info = SymInfo::new(&sym, member);
        match syms.get(&info.name) {
            Some(existing) => {
                dups.push(info);
                dups.push(existing.clone());
            }
            None => {
                syms.insert(info.name.clone(), info);
            }
        }
    });

    if cfg!(windows) {
        let allowed_dup_pfx = ["__real@", "__xmm@"];
        dups.retain(|sym| !allowed_dup_pfx.iter().any(|pfx| sym.name.starts_with(pfx)));
    }

    if !dups.is_empty() {
        dups.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        panic!("Found duplicate symbols: {dups:#?}");
    }

    println!("success: no duplicate symbols found");
}

fn verify_core_symbols(path: impl AsRef<Path>) {
    println!(
        "Checking for references to core at {}",
        path.as_ref().display()
    );

    let mut defined = BTreeSet::new();
    let mut undefined = Vec::new();

    for_each_symbol(path, |sym, member| {
        if !sym.name().unwrap().contains("_ZN4core") {
            return;
        }

        let info = SymInfo::new(&sym, member);
        if info.is_undefined {
            undefined.push(info);
        } else {
            defined.insert(info.name);
        }
    });

    undefined.retain(|sym| !defined.contains(&sym.name));

    if !undefined.is_empty() {
        undefined.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        panic!("Found undefined symbols from `core`: {undefined:#?}");
    }

    println!("success: no undefined references to core found");
}

/// For a given archive path, do something with each symbol.
fn for_each_symbol(path: impl AsRef<Path>, mut f: impl FnMut(Symbol, &ArchiveMember)) {
    let archive_data = fs::read(path).expect("reading file failed");
    let x = ArchiveFile::parse(archive_data.as_slice()).expect("archive parse failed");
    for member in x.members() {
        let member = member.unwrap();
        let data = member.data(&*archive_data).unwrap();
        let obj = object::File::parse(data).expect("object parse failed");

        for sym in obj.symbols() {
            f(sym, &member);
        }
    }
}
