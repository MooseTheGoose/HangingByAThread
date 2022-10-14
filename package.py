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
        os.mkdir(name)
    except FileExistsError:
        pass
def unlink_f(name):
    try:
        os.unlink(name)
    except FileNotFoundError:
        pass

def main():
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
        abi_root = os.path.join('target', 'androabis')
        mkdir_p(abi_root)
        for target, abi in android_abis:
            targetPath = os.path.join('target', target, 'debug')
            abiPath = os.path.join(abi_root, abi)
            mkdir_p(abiPath)
            for fname in os.listdir(targetPath):
                if fname.startswith('libhbat'):
                    abiLink = os.path.join(abiPath, fname)
                    unlink_f(abiLink)
                    os.link(os.path.join(targetPath, fname), os.path.join(abiPath, fname))
    # TODO: Build Apple package eventually
    return 0

if __name__=='__main__':
    sys.exit(main())
