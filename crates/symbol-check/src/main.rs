//! Tool used by CI to inspect compiler-builtins archives and help ensure we won't run into any
//! linking errors.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use object::elf::SHF_EXECINSTR;
use object::read::archive::{ArchiveFile, ArchiveMember};
use object::{
    File as ObjFile, Object, ObjectSection, ObjectSymbol, SectionFlags, Symbol, SymbolKind,
    SymbolScope, SymbolSection,
};
use serde_json::Value;

const CHECK_LIBRARIES: &[&str] = &["compiler_builtins", "builtins_test_intrinsics"];
const CHECK_EXTENSIONS: &[Option<&str>] = &[Some("rlib"), Some("a"), Some("exe"), None];

const USAGE: &str = "Usage:

    symbol-check build-and-check CARGO_ARGS ...

Cargo will get invoked with `CARGO_ARGS` and all output
`compiler_builtins*.rlib` files will be checked.
";

fn main() {
    // Create a `&str` vec so we can match on it.
    let args = std::env::args().collect::<Vec<_>>();
    let args_ref = args.iter().map(String::as_str).collect::<Vec<_>>();

    match &args_ref[1..] {
        ["build-and-check", "--target", target, args @ ..] if !args.is_empty() => {
            run_build_and_check(Some(target), args);
        }
        ["build-and-check", args @ ..] if !args.is_empty() => {
            run_build_and_check(None, args);
        }
        _ => {
            println!("{USAGE}");
            std::process::exit(1);
        }
    }
}

fn run_build_and_check(target: Option<&str>, args: &[&str]) {
    let paths = exec_cargo_with_args(target, args);
    for path in paths {
        println!("Checking {}", path.display());
        let archive = Archive::from_path(&path);

        verify_no_duplicates(&archive);
        verify_core_symbols(&archive);
        verify_no_exec_stack(&archive);
    }
}

fn host_target() -> String {
    let out = Command::new("rustc")
        .arg("--version")
        .arg("--verbose")
        .output()
        .unwrap();
    assert!(out.status.success());
    let out = String::from_utf8(out.stdout).unwrap();
    out.lines()
        .find_map(|s| s.strip_prefix("host: "))
        .unwrap()
        .to_owned()
}

/// Run `cargo build` with the provided additional arguments, collecting the list of created
/// libraries.
fn exec_cargo_with_args(target: Option<&str>, args: &[&str]) -> Vec<PathBuf> {
    let mut host = String::new();
    let target = target.unwrap_or_else(|| {
        host = host_target();
        host.as_str()
    });

    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--target", target, "--message-format=json"])
        .args(args)
        .stdout(Stdio::piped());

    println!("running: {cmd:?}");
    let mut child = cmd.spawn().expect("failed to launch Cargo");

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut check_files = Vec::new();

    for line in reader.lines() {
        let line = line.expect("failed to read line");
        println!("{line}"); // tee to stdout

        // Select only steps that create files
        let j: Value = serde_json::from_str(&line).expect("failed to deserialize");
        if j["reason"] != "compiler-artifact" {
            continue;
        }

        // Find rlibs in the created file list that match our expected library names and
        // extensions.
        for fpath in j["filenames"].as_array().expect("filenames not an array") {
            let path = fpath.as_str().expect("file name not a string");
            let path = PathBuf::from(path);

            if CHECK_EXTENSIONS.contains(&path.extension().map(|ex| ex.to_str().unwrap())) {
                let fname = path.file_name().unwrap().to_str().unwrap();

                if CHECK_LIBRARIES.iter().any(|lib| fname.contains(lib)) {
                    check_files.push(path);
                }
            }
        }
    }

    assert!(child.wait().expect("failed to wait on Cargo").success());

    assert!(!check_files.is_empty(), "no compiler_builtins rlibs found");
    println!("Collected the following rlibs to check: {check_files:#?}");

    check_files
}

/// Information collected from `object`, for convenience.
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
///
/// Note that this will also locate cases where a symbol is weakly defined in more than one place.
/// Technically there are no linker errors that will come from this, but it keeps our binary more
/// straightforward and saves some distribution size.
fn verify_no_duplicates(archive: &Archive) {
    let mut syms = BTreeMap::<String, SymInfo>::new();
    let mut dups = Vec::new();
    let mut found_any = false;

    archive.for_each_symbol(|symbol, member| {
        // Only check defined globals
        if !symbol.is_global() || symbol.is_undefined() {
            return;
        }

        let sym = SymInfo::new(&symbol, member);

        // x86-32 includes multiple copies of thunk symbols
        if sym.name.starts_with("__x86.get_pc_thunk") {
            return;
        }

        // Windows has symbols for literal numeric constants, string literals, and MinGW pseudo-
        // relocations. These are allowed to have repeated definitions.
        let win_allowed_dup_pfx = ["__real@", "__xmm@", "??_C@_", ".refptr"];
        if win_allowed_dup_pfx
            .iter()
            .any(|pfx| sym.name.starts_with(pfx))
        {
            return;
        }

        match syms.get(&sym.name) {
            Some(existing) => {
                dups.push(sym);
                dups.push(existing.clone());
            }
            None => {
                syms.insert(sym.name.clone(), sym);
            }
        }

        found_any = true;
    });

    assert!(found_any, "no symbols found");

    if !dups.is_empty() {
        dups.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        panic!("found duplicate symbols: {dups:#?}");
    }

    println!("    success: no duplicate symbols found");
}

/// Ensure that there are no references to symbols from `core` that aren't also (somehow) defined.
fn verify_core_symbols(archive: &Archive) {
    let mut defined = BTreeSet::new();
    let mut undefined = Vec::new();
    let mut has_symbols = false;

    archive.for_each_symbol(|symbol, member| {
        has_symbols = true;

        // Find only symbols from `core`
        if !symbol.name().unwrap().contains("_ZN4core") {
            return;
        }

        let sym = SymInfo::new(&symbol, member);
        if sym.is_undefined {
            undefined.push(sym);
        } else {
            defined.insert(sym.name);
        }
    });

    assert!(has_symbols, "no symbols found");

    // Discard any symbols that are defined somewhere in the archive
    undefined.retain(|sym| !defined.contains(&sym.name));

    if !undefined.is_empty() {
        undefined.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        panic!("found undefined symbols from core: {undefined:#?}");
    }

    println!("    success: no undefined references to core found");
}

/// Check that all object files contain a section named `.note.GNU-stack`, indicating a
/// nonexecutable stack.
fn verify_no_exec_stack(archive: &Archive) {
    let mut problem_objfiles = Vec::new();

    archive.for_each_object(|obj, member| {
        if obj_has_exe_stack(&obj) {
            problem_objfiles.push(String::from_utf8_lossy(member.name()).into_owned());
        }
    });

    if !problem_objfiles.is_empty() {
        panic!(
            "the following archive members have executable sections but no \
            `.note.GNU-stack` section: {problem_objfiles:#?}"
        );
    }

    println!("    success: no writeable-executable sections found");
}

fn obj_has_exe_stack(obj: &ObjFile) -> bool {
    // Files other than elf likely do not use the same convention.
    if !matches!(obj, ObjFile::Elf32(_) | ObjFile::Elf64(_)) {
        return false;
    }

    let mut has_exe_sections = false;
    for sec in obj.sections() {
        let SectionFlags::Elf { sh_flags } = sec.flags() else {
            unreachable!("only elf files are being checked");
        };

        let exe = (sh_flags & SHF_EXECINSTR as u64) != 0;
        has_exe_sections |= exe;

        // Located a GNU-stack section, nothing else to do
        if sec.name().unwrap_or_default() == ".note.GNU-stack" {
            return false;
        }
    }

    // Ignore object files that have no executable sections, like rmeta
    if !has_exe_sections {
        return false;
    }

    true
}

/// Thin wrapper for owning data used by `object`.
struct Archive {
    data: Vec<u8>,
}

impl Archive {
    fn from_path(path: &Path) -> Self {
        Self {
            data: fs::read(path).expect("reading file failed"),
        }
    }

    fn file(&self) -> ArchiveFile<'_> {
        ArchiveFile::parse(self.data.as_slice()).expect("archive parse failed")
    }

    /// For a given archive, do something with each object file.
    fn for_each_object(&self, mut f: impl FnMut(ObjFile, &ArchiveMember)) {
        let archive = self.file();

        for member in archive.members() {
            let member = member.expect("failed to access member");
            let obj_data = member
                .data(self.data.as_slice())
                .expect("failed to access object");
            let obj = ObjFile::parse(obj_data).expect("failed to parse object");
            f(obj, &member);
        }
    }

    /// For a given archive, do something with each symbol.
    fn for_each_symbol(&self, mut f: impl FnMut(Symbol, &ArchiveMember)) {
        self.for_each_object(|obj, member| {
            obj.symbols().for_each(|sym| f(sym, member));
        });
    }
}

#[test]
fn check_obj() {
    let Some(p) = option_env!("HAS_WX_OBJ") else {
        return;
    };

    let f = fs::read(p).unwrap();
    let obj = ObjFile::parse(f.as_slice()).unwrap();
    assert!(obj_has_exe_stack(&obj));
}
