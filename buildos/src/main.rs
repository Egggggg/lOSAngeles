// The original version of this file can be found at https://git.ablecorp.us/AbleOS/ableos/src/branch/master/repbuild/src/main.rs

use {
    error_stack::{bail, report, Context, IntoReport, Result, ResultExt},
    fatfs::{FileSystem, FormatVolumeOptions, FsOptions, ReadWriteSeek},
    std::{fmt::Display, fs::File, io, path::Path, process::Command},
};

// change the part before `,ac97` to what you want qemu to use
const AUDIO_DRIVER: &'static str = "alsa";

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut args = std::env::args();
    args.next();

    match args.next().as_deref() {
        Some("build" | "b") => {
            let mut release = false;
            for arg in args {
                if arg == "-r" || arg == "--release" {
                    release = true;
                }
            }

            build(release).change_context(Error::Build)
        }
        Some("run" | "r") => {
            let mut release = false;
            for arg in args {
                if arg == "-r" || arg == "--release" {
                    release = true;
                }
            }

            build(release)?;
            run()
        }
        Some("help" | "h") => {
            println!(concat!(
                "lOSAngeles RepBuild\n",
                "Subcommands:\n",
                "  build (b): Build a bootable disk image\n",
                "   help (h): Print this message\n",
                "    run (r): Build and run lOSAngeles in QEMU\n\n",
                "Options for build and run:\n",
                "        -r: build in release mode",
            ),);
            Ok(())
        }
        _ => Err(report!(Error::InvalidSubCom)),
    }
}

fn get_fs() -> Result<FileSystem<impl ReadWriteSeek>, io::Error> {
    let path = Path::new("target/disk.img");

    match std::fs::metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        Err(e) => bail!(e),
        Ok(_) => {
            return FileSystem::new(
                File::options().read(true).write(true).open(path)?,
                FsOptions::new(),
            )
            .into_report()
        }
    }

    let mut img = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    img.set_len(1024 * 1024 * 64)?;

    fatfs::format_volume(&mut img, FormatVolumeOptions::new())?;

    let fs = FileSystem::new(img, FsOptions::new())?;
    let bootdir = fs.root_dir().create_dir("efi")?.create_dir("boot")?;

    io::copy(
        &mut File::open("limine/BOOTX64.EFI")
            .into_report()
            .attach_printable("copying Limine bootloader (have you pulled the submodule?)")?,
        &mut bootdir.create_file("BOOTX64.EFI")?,
    )?;

    io::copy(
        &mut File::open("buildos/limine.cfg")?,
        &mut fs.root_dir().create_file("limine.cfg")?,
    )?;

    io::copy(
        &mut File::open("limine/limine.sys")?,
        &mut fs.root_dir().create_file("limine.sys")?,
    )?;

    drop(bootdir);
    Ok(fs)
}

fn build(release: bool) -> Result<(), Error> {
    let fs = get_fs().change_context(Error::Io)?;
    let mut com = Command::new("cargo");
    com.current_dir("kernel");
    com.args(["build"]);

    if release {
        com.arg("--release");
    }

    match com.status() {
        Ok(s) if s.code() != Some(0) => bail!(Error::Build),
        Err(e) => bail!(report!(e).change_context(Error::Build)),
        _ => (),
    }

    (|| -> std::io::Result<_> {
        io::copy(
            &mut File::open(
                Path::new("target/x86_64-angeles")
                    .join(if release { "release" } else { "debug" })
                    .join("losangeles.elf"),
            )?,
            &mut fs.root_dir().create_file("losangeles.elf")?,
        )
        .map(|_| ())
    })()
    .into_report()
    .change_context(Error::Io)
}

fn run() -> Result<(), Error> {
    let mut com = Command::new("qemu-system-x86_64");

    com.args([
        "-serial", "stdio",
        "-m", "512m",
        "-cpu", "qemu64",
        "-hda", "disk.img",
        "-audio", &format!("{AUDIO_DRIVER},model=ac97"),
        "-vga", "cirrus",
    ]);

    match com
        .status()
        .into_report()
        .change_context(Error::ProcessSpawn)?
    {
        s if s.success() => Ok(()),
        s => Err(report!(Error::Qemu(s.code()))),
    }
}

#[derive(Debug)]
enum Error {
    Build,
    InvalidSubCom,
    Io,
    ProcessSpawn,
    Qemu(Option<i32>),
}

impl Context for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Build => f.write_str("failed to build the kernel"),
            Self::InvalidSubCom => {
                f.write_str("missing or invalid subcommand (available: build, run)")
            }
            Self::Io => f.write_str("IO error"),
            Self::ProcessSpawn => f.write_str("failed to spawn a process"),
            Self::Qemu(Some(c)) => write!(f, "QEMU Error: {c}"),
            Self::Qemu(None) => write!(f, "QEMU Error: interrupted by signal"),
        }
    }
}