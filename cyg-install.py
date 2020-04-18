import argparse
from pathlib import Path
import shutil
import sys
from textwrap import dedent
import winreg

WRAPPER = Path(__file__).parent / "target/release/cyg.exe"

def install_wrapper(target, name=None):
    if name is None:
        name = "cyg"
    target = (target / name).with_suffix(".exe")
    shutil.copyfile(WRAPPER, target)

def install_config(target, cygdir=None):
    if cygdir is None:
        cygdir = find_cygwin()
    if cygdir is None:
        raise SystemExit("Unable to locate cygwin. Please specify.")
    toml = dedent(f"""\
    base = '{cygdir}'
    """)
    config_file = Path(target) / "cygwin.toml"
    config_file.write_text(toml, encoding="utf-8")

def find_cygwin():
    try:
        with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, "Software\\Cygwin\\setup") as k:
            return winreg.QueryValueEx(k, "rootdir")[0]
    except FileNotFoundError:
        return None

def make_parser():
    parser = argparse.ArgumentParser(description="Install the cygwin wrapper utility cyg")
    parser.add_argument("target",
        help="The directory where the wrappers should be installed")
    parser.add_argument("--commands", "-c", metavar="CMD", nargs="+",
        help="Individual commands to make wrappers for")
    return parser

if __name__ == "__main__":
    parser = make_parser()
    args = parser.parse_args()
    target = Path(args.target)
    if target.exists() and not target.is_dir():
        raise SystemExit("Please specify a directory")
    if not target.exists():
        target.mkdir(parents=True)
    install_config(target)
    install_wrapper(target)
    for cmd in args.commands:
        install_wrapper(target, cmd)
