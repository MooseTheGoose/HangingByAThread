#!/usr/bin/env python3
import sys
import os
import subprocess
import shutil

ANDROID_ABI_FILTER_LUT = {
    'aarch64-linux-android': 'arm64-v8a',
    'x86_64-linux-android': 'x86_64',
    'i686-linux-android': 'x86',
    'armv7-linux-androideabi': 'armeabi-v7a',
}

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

def build_rust():
    target_arches = {
        'aarch64-linux-android',
        'armv7-linux-androideabi',
        'i686-linux-android',
        'x86_64-linux-android'
    }
    # Parse targets from rustup to see what is installed
    p = subprocess.run(['rustup', 'target', 'list'], capture_output=True)
    installed_targets = set()
    for l in p.stdout.splitlines():
        target_installed = l.decode().split()
        if len(target_installed) > 1 and 'installed' in target_installed[1]:
            installed_targets.add(target_installed[0])
    # Install targets then build with cargo
    for target in target_arches:
        if target not in installed_targets:
            subprocess.run(['rustup', 'target', 'install', target])
        subprocess.run(['cargo', 'build', '--target', target])
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
