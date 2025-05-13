//! Tool used by CI to inspect compiler-builtins archives and help ensure we won't run into any
//! linking errors.

use object::read::archive::{ArchiveFile, ArchiveMember};
use object::{Object, ObjectSymbol, Symbol, SymbolKind, SymbolScope, SymbolSection};
use std::collections::{BTreeMap, BTreeSet};
use std::{fs, path::Path};

const USAGE: &str = "Usage:

    symbol-check check-duplicates ARCHIVE ...
    symbol-check check-core-syms ARCHIVE ...

Note that multiple archives may be specified but they are checked independently
rather than as a group.";

fn main() {
    // Create a `&str` vec so we can match on it.
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
    println!("Checking `{}` for duplicates", path.as_ref().display());

    let mut syms = BTreeMap::<String, SymInfo>::new();
    let mut dups = Vec::new();

    for_each_symbol(path, |sym, member| {
        // Only check defined globals, exclude wasm file symbols
        if !sym.is_global() || sym.is_undefined() || sym.kind() == SymbolKind::File {
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
        // Ignore literal constants
        let allowed_dup_pfx = ["__real@", "__xmm@"];
        dups.retain(|sym| !allowed_dup_pfx.iter().any(|pfx| sym.name.starts_with(pfx)));
    }

    if !dups.is_empty() {
        dups.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        panic!("found duplicate symbols: {dups:#?}");
    }

    println!("success: no duplicate symbols found");
}

/// Ensure that there are no references to symbols from `core` that aren't also (somehow) defined.
fn verify_core_symbols(path: impl AsRef<Path>) {
    println!(
        "Checking `{}` for references to core",
        path.as_ref().display()
    );

    let mut defined = BTreeSet::new();
    let mut undefined = Vec::new();

    for_each_symbol(path, |sym, member| {
        // Find only symbols from `core`
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

    // Discard any symbols that are defined somewhere in the archive
    undefined.retain(|sym| !defined.contains(&sym.name));

    if !undefined.is_empty() {
        undefined.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        panic!("found undefined symbols from core: {undefined:#?}");
    }

    println!("success: no undefined references to core found");
}

/// For a given archive path, do something with each symbol.
fn for_each_symbol(path: impl AsRef<Path>, mut f: impl FnMut(Symbol, &ArchiveMember)) {
    let data = fs::read(path).expect("reading file failed");
    let archive = ArchiveFile::parse(data.as_slice()).expect("archive parse failed");
    for member in archive.members() {
        let member = member.expect("failed to access member");
        let obj_data = member.data(&*data).expect("failed to access object");
        let obj = object::File::parse(obj_data).expect("failed to parse object");
        obj.symbols().for_each(|sym| f(sym, &member));
    }
}
