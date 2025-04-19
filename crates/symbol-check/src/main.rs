#![allow(unused)]

// use ar::Archive;
use object::read::archive::{ArchiveFile, ArchiveMember};
use object::read::elf::FileHeader;
use object::{Object, ObjectSection, ObjectSymbol, Symbol, SymbolKind, SymbolScope};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

const USAGE: &str = "Usage:
    symbol-check check-duplicates PATH...
    symbol-check check-core-syms PATH...
";

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let args_ref = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();

    match &args_ref[1..] {
        ["check-duplicates", rest @ ..] if !rest.is_empty() => {
            rest.iter().for_each(verify_no_duplicates)
        }
        ["check-core-syms", rest @ ..] if !rest.is_empty() => {
            rest.iter().for_each(verify_no_duplicates)
        }
        _ => {
            println!("{USAGE}");
            std::process::exit(1);
        }
    }

    // Raise an error if the same symbol is present in multiple object files
}

#[derive(Debug, Clone)]
struct SymInfo {
    name: String,
    kind: SymbolKind,
    scope: SymbolScope,
    address: u64,
    is_weak: bool,
    object: String,
}

impl SymInfo {
    fn new(sym: Symbol, member: &ArchiveMember) -> Self {
        Self {
            name: sym.name().expect("missing name").to_owned(),
            kind: sym.kind(),
            scope: sym.scope(),
            address: sym.address(),
            is_weak: sym.is_weak(),
            object: String::from_utf8_lossy(member.name()).into_owned(),
        }
    }
}

fn verify_no_duplicates(path: impl AsRef<Path>) {
    println!("Checking for duplicates at {:?}", path.as_ref());

    // Global defined symbols
    let mut syms = BTreeMap::<String, SymInfo>::new();
    let mut dups = Vec::new();

    for_each_symbol(path, |sym, member| {
        if sym.is_global() && !sym.is_undefined() {
            let info = SymInfo::new(sym, member);
            match syms.get(&info.name) {
                Some(existing) => {
                    dups.push(info);
                    dups.push(existing.clone());
                }
                None => {
                    syms.insert(info.name.clone(), info);
                }
            }
        }
        Ok(())
    })
    .unwrap();

    if !dups.is_empty() {
        dups.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        panic!("Found duplicate symbols: {dups:#?}");
    }
}

fn verify_core_symbols(path: impl AsRef<Path>) {
    todo!()
}

/// For a given archive path, do something with each symbol.
fn for_each_symbol(
    path: impl AsRef<Path>,
    mut f: impl FnMut(Symbol, &ArchiveMember) -> Result<()>,
) -> Result<()> {
    let archive_data = fs::read(path)?;
    let x = ArchiveFile::parse(archive_data.as_slice())?;
    for member in x.members() {
        let member = member.unwrap();
        let data = member.data(&*archive_data).unwrap();
        let obj = object::File::parse(data)?;

        for sym in obj.symbols() {
            f(sym, &member)?;
        }
    }

    Ok(())
}
