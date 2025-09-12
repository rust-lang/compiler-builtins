//! Tool used by CI to inspect compiler-builtins archives and help ensure we won't run into any
//! linking errors.

#![allow(unused)] // TODO

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

use object::elf::SectionHeader32;
use object::read::archive::ArchiveFile;
use object::read::elf::SectionHeader;
use object::{
    Architecture, BinaryFormat, Bytes, Endianness, File as ObjFile, LittleEndian, Object,
    ObjectSection, ObjectSymbol, Result as ObjResult, SectionFlags, SectionKind, Symbol,
    SymbolKind, SymbolScope, U32, U32Bytes, elf,
};
use serde_json::Value;

const CHECK_LIBRARIES: &[&str] = &["compiler_builtins", "builtins_test_intrinsics"];
const CHECK_EXTENSIONS: &[Option<&str>] = &[Some("rlib"), Some("a"), Some("exe"), None];

const USAGE: &str = "Usage:

    symbol-check build-and-check [TARGET] [--no-std] -- CARGO_BUILD_ARGS ...

Cargo will get invoked with `CARGO_ARGS` and the specified target. All output
`compiler_builtins*.rlib` files will be checked.

If TARGET is not specified, the host target is used.

If the `--no-std` flag is passed, the binaries will not be checked for
executable stacks under the assumption that they are not being emitted.

    check [--no-std] PATHS ...

Run the same checks on the given set of paths, without invoking Cargo. Paths
may be either archives or object files.
";

#[derive(Debug, PartialEq)]
enum Mode {
    BuildAndCheck,
    CheckOnly,
}

fn main() {
    let mut args_iter = env::args().skip(1);
    let mode = match args_iter.next() {
        Some(arg) if arg == "build-and-check" => Mode::BuildAndCheck,
        Some(arg) if arg == "check" => Mode::CheckOnly,
        Some(other) => invalid_usage(&format!("unrecognized mode `{other}`")),
        None => invalid_usage("mode must be specified"),
    };

    let mut target = None;
    let mut verify_no_exe = true;
    let mut positional = Vec::new();

    for arg in args_iter.by_ref() {
        dbg!(&arg);
        match arg.as_str() {
            "--no-std" => verify_no_exe = false,
            "--" => break,
            f if f.starts_with("-") => invalid_usage(&format!("unrecognized flag `{f}`")),
            _ if mode == Mode::BuildAndCheck => target = Some(arg),
            _ => {
                positional.push(arg);
                break;
            }
        }
    }

    positional.extend(args_iter);

    match mode {
        Mode::BuildAndCheck => {
            let target = target.unwrap_or_else(|| host_target());
            let paths = exec_cargo_with_args(&target, positional.as_slice());
            check_paths(&paths, verify_no_exe);
        }
        Mode::CheckOnly => {
            assert!(!positional.is_empty());
            check_paths(&positional, verify_no_exe);
        }
    };
}

fn invalid_usage(s: &str) -> ! {
    println!("{s}\n{USAGE}");
    std::process::exit(1);
}

fn check_paths<P: AsRef<Path>>(paths: &[P], verify_no_exe: bool) {
    for path in paths {
        let path = path.as_ref();
        println!("Checking {}", path.display());
        let archive = BinFile::from_path(path);

        // verify_no_duplicates(&archive);
        // verify_core_symbols(&archive);
        // if verify_no_exe {
        // We don't really have a good way of knowing whether or not an elf file is for a
        // no-kernel environment, in which case note.GNU-stack doesn't get emitted.
        verify_no_exec_stack(&archive);
        // }
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
fn exec_cargo_with_args<S: AsRef<str>>(target: &str, args: &[S]) -> Vec<PathBuf> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--target",
        target,
        "--message-format=json-diagnostic-rendered-ansi",
    ])
    .args(args.iter().map(|arg| arg.as_ref()))
    .stdout(Stdio::piped());

    println!("running: {cmd:?}");
    let mut child = cmd.spawn().expect("failed to launch Cargo");

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut check_files = Vec::new();

    for line in reader.lines() {
        let line = line.expect("failed to read line");
        let j: Value = serde_json::from_str(&line).expect("failed to deserialize");
        let reason = &j["reason"];

        // Forward output that is meant to be user-facing
        if reason == "compiler-message" {
            println!("{}", j["message"]["rendered"].as_str().unwrap());
        } else if reason == "build-finished" {
            println!("build finshed. success: {}", j["success"]);
        } else if reason == "build-script-executed" {
            let pretty = serde_json::to_string_pretty(&j).unwrap();
            println!("build script output: {pretty}",);
        }

        // Only interested in the artifact list now
        if reason != "compiler-artifact" {
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
    section: String,
    is_undefined: bool,
    is_global: bool,
    is_local: bool,
    is_weak: bool,
    is_common: bool,
    address: u64,
    object: String,
}

impl SymInfo {
    fn new(sym: &Symbol, obj: &ObjFile, obj_path: &str) -> Self {
        // Include the section name if possible. Fall back to the `Section` debug impl if not.
        let section = sym.section();
        let section_name = sym
            .section()
            .index()
            .and_then(|idx| obj.section_by_index(idx).ok())
            .and_then(|sec| sec.name().ok())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("{section:?}"));

        Self {
            name: sym.name().expect("missing name").to_owned(),
            kind: sym.kind(),
            scope: sym.scope(),
            section: section_name,
            is_undefined: sym.is_undefined(),
            is_global: sym.is_global(),
            is_local: sym.is_local(),
            is_weak: sym.is_weak(),
            is_common: sym.is_common(),
            address: sym.address(),
            object: obj_path.to_owned(),
        }
    }
}

/// Ensure that the same global symbol isn't defined in multiple object files within an archive.
///
/// Note that this will also locate cases where a symbol is weakly defined in more than one place.
/// Technically there are no linker errors that will come from this, but it keeps our binary more
/// straightforward and saves some distribution size.
fn verify_no_duplicates(archive: &BinFile) {
    let mut syms = BTreeMap::<String, SymInfo>::new();
    let mut dups = Vec::new();
    let mut found_any = false;

    archive.for_each_symbol(|symbol, obj, member| {
        // Only check defined globals
        if !symbol.is_global() || symbol.is_undefined() {
            return;
        }

        let sym = SymInfo::new(&symbol, obj, member);

        // x86-32 includes multiple copies of thunk symbols
        if sym.name.starts_with("__x86.get_pc_thunk") {
            return;
        }

        // GDB pretty printing symbols may show up more than once but are weak.
        if sym.section == ".debug_gdb_scripts" && sym.is_weak {
            return;
        }

        // Windows has symbols for literal numeric constants, string literals, and MinGW pseudo-
        // relocations. These are allowed to have repeated definitions.
        let win_allowed_dup_pfx = ["__real@", "__xmm@", "__ymm@", "??_C@_", ".refptr"];
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
fn verify_core_symbols(archive: &BinFile) {
    let mut defined = BTreeSet::new();
    let mut undefined = Vec::new();
    let mut has_symbols = false;

    archive.for_each_symbol(|symbol, obj, member| {
        has_symbols = true;

        // Find only symbols from `core`
        if !symbol.name().unwrap().contains("_ZN4core") {
            return;
        }

        let sym = SymInfo::new(&symbol, obj, member);
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

/// Ensure that the object/archive will not require an executable stack.
fn verify_no_exec_stack(archive: &BinFile) {
    let mut problem_objfiles = Vec::new();

    archive.for_each_object(|obj, obj_path| {
        if obj_requires_exe_stack(&obj) {
            problem_objfiles.push(obj_path.to_owned());
        }
    });

    if !problem_objfiles.is_empty() {
        panic!("the following object files require an executable stack: {problem_objfiles:#?}");
    }

    println!("    success: no writeable+executable sections found");
}

/// True if the section/flag combination indicates that the object file should be linked with an
/// executable stack.
///
/// Paraphrased from <https://www.man7.org/linux/man-pages/man1/ld.1.html>:
///
/// - A `.note.GNU-stack` section with the exe flag means this needs an executable stack
/// - A `.note.GNU-stack` section without the exe flag means there is no executable stack needed
/// - Without the section, behavior is target-specific and on some targets means an executable
///   stack is required.
///
/// If any object files meet the requirements for an executable stack, any final binary that links
/// it will have a program header with a `PT_GNU_STACK` section, which will be marked `RWE` rather
/// than the desired `RW`. (We don't check final binaries).
///
/// Per [1], it is now deprecated behavior for a missing `.note.GNU-stack` section to imply an
/// executable stack. However, we shouldn't assume that tooling has caught up to this.
///
/// [1]: https://sourceware.org/git/gitweb.cgi?p=binutils-gdb.git;h=0d38576a34ec64a1b4500c9277a8e9d0f07e6774>
fn obj_requires_exe_stack(obj: &ObjFile) -> bool {
    // Files other than elf do not use the same convention.
    if obj.format() != BinaryFormat::Elf {
        return false;
    }

    let secs = match obj {
        ObjFile::Elf32(elf_file) => elf_file.sections(),
        ObjFile::Elf64(elf_file) => panic!(),
        // ObjFile::Elf64(elf_file) => elf_file.sections(),
        _ => return false,
    };

    let mut return_immediate = None;
    let mut has_exe_sections = false;
    for sec in obj.sections() {
        dbg!(sec.name());
        let SectionFlags::Elf { sh_flags } = sec.flags() else {
            unreachable!("only elf files are being checked");
        };

        if sec.kind() == SectionKind::Elf(elf::SHT_ARM_ATTRIBUTES) {
            let end = obj.endianness();
            let data = sec.data().unwrap();
            let ObjFile::Elf32(elf) = obj else { panic!() };
            let elf_sec = elf.section_by_index(sec.index()).unwrap();
            let elf_hdr = elf_sec.elf_section_header();

            parse_arm_thing(data, elf_hdr, end);
        }

        let is_exe = (sh_flags & elf::SHF_EXECINSTR as u64) != 0;

        // If the magic section is present, its exe bit tells us whether or not the object
        // file requires an executable stack.
        if sec.name().unwrap_or_default() == ".note.GNU-stack" {
            return_immediate = Some(is_exe);
        }

        // Otherwise, just keep track of whether or not we have exeuctable sections
        has_exe_sections |= is_exe;
    }

    if let Some(imm) = return_immediate {
        return imm;
    }

    // Ignore object files that have no executable sections, like rmeta
    if !has_exe_sections {
        return false;
    }

    platform_default_exe_stack_required(obj.architecture(), obj.endianness())
}

/// Default if there is no `.note.GNU-stack` section.
fn platform_default_exe_stack_required(arch: Architecture, end: Endianness) -> bool {
    match arch {
        // PPC64 doesn't set `.note.GNU-stack` since GNU nested functions don't need a trampoline,
        // <https://gcc.gnu.org/bugzilla/show_bug.cgi?id=21098>.
        Architecture::PowerPc64 if end == Endianness::Big => false,
        _ => true,
    }
}

// See https://github.com/ARM-software/abi-aa/blob/main/addenda32/addenda32.rst#33public-aeabi-attribute-tags
fn parse_arm_thing(data: &[u8], elf_hdr: &SectionHeader32<Endianness>, end: Endianness) {
    let attrs = elf_hdr.attributes(end, data).unwrap();
    dbg!(attrs);

    eprintln!("data d: {data:?}");
    eprintln!("data x: {data:x?}");
    eprintln!("data string: {:?}", String::from_utf8_lossy(data));
    // eprintln!("data: {:x?}", &data[16..]);
    // let mut rest = &data[16..];
    let mut b = Bytes(data);
    let _fmt_version = b.read::<u8>().unwrap();
    let _sec_length = b.read::<U32<LittleEndian>>().unwrap();

    // loop {
    let s = b.read_string().unwrap();
    eprintln!("abi {}", String::from_utf8_lossy(s));

    let _tag = b.read_uleb128().unwrap();
    let _size = b.read::<U32<LittleEndian>>().unwrap();

    // NUL-terminated byte strings
    const CPU_RAW_NAME: u64 = 4;
    const CPU_NAME: u64 = 5;
    const ALSO_COMPATIBLE_WITH: u64 = 65;
    const CONFORMANCE: u64 = 67;

    const CPU_ARCH_PROFILE: u64 = 7;

    while !b.is_empty() {
        let tag = b.read_uleb128().unwrap();
        match tag {
            CONFORMANCE => eprintln!(
                "conf: {}",
                String::from_utf8_lossy(b.read_string().unwrap())
            ),
            // 77 =>
            CPU_ARCH_PROFILE => {
                // CPU_arch_profile
                let value = b.read_uleb128().unwrap();
            }
            _ => eprintln!("tag {tag} value {}", b.read::<u8>().unwrap()),
        }
    }

    // }

    // while !rest.is_empty() {}
}

/// Thin wrapper for owning data used by `object`.
struct BinFile {
    path: PathBuf,
    data: Vec<u8>,
}

impl BinFile {
    fn from_path(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
            data: fs::read(path).expect("reading file failed"),
        }
    }

    fn as_archive_file(&self) -> ObjResult<ArchiveFile<'_>> {
        ArchiveFile::parse(self.data.as_slice())
    }

    fn as_obj_file(&self) -> ObjResult<ObjFile<'_>> {
        ObjFile::parse(self.data.as_slice())
    }

    /// For a given archive, do something with each object file. For an object file, do
    /// something once.
    fn for_each_object(&self, mut f: impl FnMut(ObjFile, &str)) {
        // Try as an archive first.
        let as_archive = self.as_archive_file();
        if let Ok(archive) = as_archive {
            for member in archive.members() {
                let member = member.expect("failed to access member");
                let obj_data = member
                    .data(self.data.as_slice())
                    .expect("failed to access object");
                let obj = ObjFile::parse(obj_data).expect("failed to parse object");
                f(obj, &String::from_utf8_lossy(member.name()));
            }

            return;
        }

        // Fall back to parsing as an object file.
        let as_obj = self.as_obj_file();
        if let Ok(obj) = as_obj {
            f(obj, &self.path.to_string_lossy());
            return;
        }

        panic!(
            "failed to parse as either archive or object file: {:?}, {:?}",
            as_archive.unwrap_err(),
            as_obj.unwrap_err(),
        );
    }

    /// D something with each symbol in an archive or object file.
    fn for_each_symbol(&self, mut f: impl FnMut(Symbol, &ObjFile, &str)) {
        self.for_each_object(|obj, obj_path| {
            obj.symbols().for_each(|sym| f(sym, &obj, obj_path));
        });
    }
}

/// Check with a binary that has no `.note.GNU-stack` section, indicating platform-default stack
/// writeability.
#[test]
fn check_no_gnu_stack_obj() {
    // Should be supported on all Unix platforms
    let p = env!("NO_GNU_STACK_OBJ");
    let f = fs::read(p).unwrap();
    let obj = ObjFile::parse(f.as_slice()).unwrap();
    dbg!(
        obj.format(),
        obj.architecture(),
        obj.sub_architecture(),
        obj.is_64()
    );
    let has_exe_stack = obj_requires_exe_stack(&obj);

    let obj_target = env!("OBJ_TARGET");
    if obj_target.contains("-windows-") || obj_target.contains("-apple-") {
        // Non-ELF targets don't have executable stacks marked in the same way
        assert!(!has_exe_stack);
    } else {
        assert!(has_exe_stack);
    }
}

#[test]
#[cfg_attr(
    any(target_os = "windows", target_vendor = "apple"),
    ignore = "requires elf format"
)]
fn check_obj() {
    #[expect(clippy::option_env_unwrap, reason = "test is ignored")]
    let p = option_env!("HAS_EXE_STACK_OBJ").expect("has_exe_stack.o not present");
    let f = fs::read(p).unwrap();
    let obj = ObjFile::parse(f.as_slice()).unwrap();
    dbg!(
        obj.format(),
        obj.architecture(),
        obj.sub_architecture(),
        obj.is_64()
    );
    assert!(obj_requires_exe_stack(&obj));
}
