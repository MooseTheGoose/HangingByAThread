import sys
import os
import subprocess

PACKAGE_NAME='com.example.native_activity'

def install_apk():
    debug_apk = os.path.join(
        'platform', 'Android', 'app', 'build',
        'outputs', 'apk', 'debug', 'app-debug.apk')
    subprocess.run(['adb', 'install', debug_apk])
    subprocess.run(['adb', 'shell',
        'am', 'start', '-D', '{}/{}.HBATActivity'.format(PACKAGE_NAME, PACKAGE_NAME)])

def get_package_pid(retries=10):
    pid = None
    i = 0
    while pid is None and i < retries:
        ps = subprocess.run(['adb', 'shell',
            'run-as', PACKAGE_NAME, 'ps', '-A'], capture_output=True).stdout
        for binLine in ps.splitlines():
            args = binLine.decode().split()
            if len(args) > 8 and args[8] == PACKAGE_NAME:
                pid = int(args[1])
                break
        i += 1
    if pid is None:
        raise RuntimeError('Unable to get pid for package "{}" after trying {} times'.format(PACKAGE_NAME, retries))
    return pid

def connect_jdb(pid, port):
    subprocess.run(['adb', 'forward', 'tcp:'+str(port), 'jdwp:'+str(pid)])
    subprocess.run(['jdb', '-connect', 'com.sun.jdi.SocketAttach:hostname=localhost,port='+str(port)])

def get_target_arch():
    mach = subprocess.run(['adb', 'shell', 'uname', '-m'], capture_output=True).stdout.decode()
    if 'aarch64' in mach or 'armv8' in mach:
        return 'aarch64'
    elif 'x86_64' == mach or 'amd64' == mach:
        return 'x86_64'
    elif 'arm' in mach:
        return 'arm'
    elif 'i386' == mach or 'i686' == mach:
        return 'i386'
    else:
        raise RuntimeError('Unrecognized machine "{}" when calling "uname -m"'.format(mach))

def get_host():
    plat = sys.platform
    if plat.startswith('win'):
        return 'windows-x86_64'
    elif plat.startswith('darwin'):
        return 'darwin-x86_64'
    else:
        return 'linux-x86_64'

def connect_lldb(pid, port):
    ndk_home = os.getenv('ANDROID_NDK_HOME')
    arch = get_target_arch()
    host = get_host()
    clang_ver = '14.0.6'
    lldb_server = os.path.join(
        ndk_home, 'toolchains', 'llvm', 'prebuilt', host,
        'lib64', 'clang', clang_ver, 'lib', 'linux', arch, 'lldb-server')
    pwd = subprocess.run(['adb', 'shell',
        'run-as', PACKAGE_NAME, 'pwd'], capture_output=True).stdout.decode().strip()
    debug_sock = pwd+'/debug.sock'
    subprocess.run(['adb', 'push', lldb_server, '/data/local/tmp/lldb-server'])
    subprocess.run(['adb', 'shell',
        'chmod', '755', '/data/local/tmp/lldb-server',
        '&&', 'run-as', PACKAGE_NAME,
        'chmod', 'a+x', '.',
        '&&', 'run-as', PACKAGE_NAME,
        'cp', '/data/local/tmp/lldb-server', './lldb-server'])
    subprocess.run(['adb', 'forward', 'tcp:'+str(port), 'localfilesystem:'+debug_sock])
    subprocess.run(['adb', 'shell',
        'run-as', PACKAGE_NAME,
        'rm', '-f', debug_sock,
        '&&', 'run-as', PACKAGE_NAME,
        './lldb-server', 'g', 'unix://'+debug_sock,
        '--attach', str(pid)])

def main():
    debug_port = 5930
    lldb_port = 3059
    install_apk()
    pid = get_package_pid()
    connect_jdb(pid, debug_port)
    connect_lldb(pid, lldb_port) 
    return 0

if __name__=='__main__':
    sys.exit(main())
