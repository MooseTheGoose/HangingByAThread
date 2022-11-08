#!/usr/bin/env python3
import sys
import os
import platform
import subprocess
import shutil

ANDROID_ABI_FILTER_LUT = {
    'aarch64-linux-android': 'arm64-v8a',
    'x86_64-linux-android': 'x86_64',
    'i686-linux-android': 'x86',
    'armv7-linux-androideabi': 'armeabi-v7a',
}
ANDROID_TARGET_ARCHES = {
    'aarch64-linux-android',
    'armv7-linux-androideabi',
    'i686-linux-android',
    'x86_64-linux-android',
}
ANDROID_API_VERSION = 33

def mkdir_p(name):
    try:
        os.makedirs(name)
    except FileExistsError:
        pass

def unlink_f(name):
    try:
        os.unlink(name)
    except FileNotFoundError:
        pass

def mklink_f(fname, linkname):
    unlink_f(linkname)
    os.link(fname, linkname)

def android_toolchain_dir():
    return os.path.join(
        get_ndk_home(), 'toolchains', 'llvm', 'prebuilt',
        platform.system().lower()+'-x86_64', 'bin')

def android_tool_prefix(target, version):
    if target == 'armv7-linux-androideabi':
        target = 'armv7a-linux-androideabi'
    return f'{target}{version}-'

def android_gen_config(target_arches,version):
    toml_text = ''
    toml_file = os.path.join('.cargo', 'config.toml')
    mkdir_p('.cargo')
    if os.path.exists(toml_file):
        with open(toml_file) as fp:
            toml_text = fp.read()
    toolchain_dir = android_toolchain_dir()
    new_toml_text = toml_text
    for t in target_arches:
        # Not particularly robust but good enough
        if t not in toml_text:
            tool_prefix = android_tool_prefix(t,version)
            linker_name = f'{tool_prefix}clang++'
            new_toml_text += f'[target.{t}]\n'
            new_toml_text += f'linker = "{os.path.join(toolchain_dir, linker_name)}"\n'
            new_toml_text += f'rustflags = ["-C", "link-arg=-L{os.path.join(os.getcwd(), "libgcc-shim")}"]\n'
    if new_toml_text != toml_text:
        with open(toml_file, 'w') as fp:
            fp.write(new_toml_text)

def get_ndk_home():
    env = os.getenv('ANDROID_NDK_HOME')
    if env is None:
        env = os.getenv('NDK_HOME')
    if env is None:
        raise RuntimeError('You must define the environment variable ANDROID_NDK_HOME to build with Android')
    return env

def build_rust():
    target_arches = set(ANDROID_TARGET_ARCHES)
    # Parse targets from rustup to see what is installed
    p = subprocess.run(['rustup', 'target', 'list'], capture_output=True)
    installed_targets = set()
    for l in p.stdout.splitlines():
        target_installed = l.decode().split()
        if len(target_installed) > 1 and 'installed' in target_installed[1]:
            installed_targets.add(target_installed[0])
    # Generate cargo.toml file depending on targets
    android_gen_config(target_arches, ANDROID_API_VERSION)
    # Install targets then build with cargo
    for target in target_arches:
        if target not in installed_targets:
            subprocess.check_call(['rustup', 'target', 'install', target])
        subprocess.check_call(['cargo', 'build', '--target', target])
    # Build android package using targets specified
    android_abis = [(target, ANDROID_ABI_FILTER_LUT[target]) for target in target_arches if 'android' in target]
    if len(android_abis) > 0:
        abi_root = os.path.join('build', 'androabis')
        mkdir_p(abi_root)
        for target, abi in android_abis:
            targetPath = os.path.join('target', target, 'debug')
            abiPath = os.path.join(abi_root, abi)
            mkdir_p(abiPath)
            for fname in os.listdir(targetPath):
                if fname.startswith('libhbat'):
                    abiLink = os.path.join(abiPath, fname)
                    mklink_f(os.path.join(targetPath, fname), abiLink)
    # TODO: Build Apple package eventually

def build_assets():
    # No assets to build at the moment.
    # They will be built when generating
    # optimized models from FBX files
    # via cargo-hbat
    pass

def main():
    build_rust()
    build_assets()
    return 0

if __name__=='__main__':
    sys.exit(main())
