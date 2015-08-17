#!/usr/bin/env python3

TEXT_PATH = 'text'
OUT_PATH = 'target'
BOOK_OUT_PATH = OUT_PATH + '/book'
STATIC_PATH = 'static'
RUSTBOOK_OUT_PATH = '_book'

import os
import re
import shutil
import subprocess
import sys
from itertools import chain

TRACE = os.environ.get('TLBORM_TRACE_BUILD', '') != ''

def main():
    build()

def build():
    sh('rustbook', 'build', TEXT_PATH)

    if os.path.exists(OUT_PATH):
        traced(really_rmtree, OUT_PATH)

    traced(shutil.copytree, RUSTBOOK_OUT_PATH, BOOK_OUT_PATH)
    traced(copy_merge, STATIC_PATH, OUT_PATH)

def copy_merge(src, dst):
    names = os.listdir(src)
    if not os.path.exists(dst): os.makedirs(dst)
    for name in names:
        srcname = os.path.join(src, name)
        dstname = os.path.join(dst, name)
        if os.path.isdir(srcname):
            copy_merge(srcname, dstname)
        else:
            shutil.copy2(srcname, dstname)
    try:
        shutil.copystat(src, dst)
    except OSError as why:
        # can't copy file access times on Windows
        if why.winerror is not None:
            raise

def sh(*cmd):
    msg_trace(' '.join(escape_argument(str(a)) for a in cmd))
    try:
        subprocess.check_call(cmd, shell=True)
    except:
        msg_trace('FAILED!')
        raise

def msg(*args):
    sys.stdout.write('> ')
    for arg in args:
        sys.stdout.write(str(arg))
    sys.stdout.write('\n')
    sys.stdout.flush()

def msg_trace(*args):
    if TRACE:
        sys.stderr.write('$ ')
        for arg in args:
            sys.stderr.write(str(arg))
        sys.stderr.write('\n')
        sys.stderr.flush()

def traced(f, *pargs, **kwargs):
    if TRACE:
        args = ', '.join(chain(
            (repr(a) for a in pargs),
            ('%s=%r' % kv for kv in kwargs.items())))
        msg_trace('%s(%s)' % (f.__qualname__, args))
    return f(*pargs, **kwargs)

def escape_argument(arg):
    # https://stackoverflow.com/questions/29213106/how-to-securely-escape-command-line-arguments-for-the-cmd-exe-shell-on-windows
    # Escape the argument for the cmd.exe shell.
    # See http://blogs.msdn.com/b/twistylittlepassagesallalike/archive/2011/04/23/everyone-quotes-arguments-the-wrong-way.aspx
    #
    # First we escape the quote chars to produce a argument suitable for
    # CommandLineToArgvW. We don't need to do this for simple arguments.

    if not arg or re.search(r'(["\s])', arg):
        arg = '"' + arg.replace('"', r'\"') + '"'

    return escape_for_cmd_exe(arg)

def escape_for_cmd_exe(arg):
    # https://stackoverflow.com/questions/29213106/how-to-securely-escape-command-line-arguments-for-the-cmd-exe-shell-on-windows
    # Escape an argument string to be suitable to be passed to
    # cmd.exe on Windows
    #
    # This method takes an argument that is expected to already be properly
    # escaped for the receiving program to be properly parsed. This argument
    # will be further escaped to pass the interpolation performed by cmd.exe
    # unchanged.
    #
    # Any meta-characters will be escaped, removing the ability to e.g. use
    # redirects or variables.
    #
    # @param arg [String] a single command line argument to escape for cmd.exe
    # @return [String] an escaped string suitable to be passed as a program
    #   argument to cmd.exe

    meta_chars = '()%!^"<>&|'
    meta_re = re.compile('(' + '|'.join(re.escape(char) for char in list(meta_chars)) + ')')
    meta_map = { char: "^%s" % char for char in meta_chars }

    def escape_meta_chars(m):
        char = m.group(1)
        return meta_map[char]

    return meta_re.sub(escape_meta_chars, arg)

def really_rmtree(path):
    WAIT_TIME_SECS = 1.0
    MAX_TRIES = 10

    def on_error(func, path, exc_info):
        """
        Error handler for ``shutil.rmtree``.

        If the error is due to an access error (read only file)
        it attempts to add write permission and then retries.

        If the error is for another reason it re-raises the error.

        Usage: ``shutil.rmtree(path, onerror=on_error)``

        From <http://stackoverflow.com/a/2656405>_.
        """
        import stat
        if not os.access(path, os.W_OK):
            # Is the error an access error ?
            os.chmod(path, stat.S_IWUSR)
            func(path)
        else:
            raise

    for _ in range(MAX_TRIES):
        failed = True
        try:
            msg_trace('shutil.rmtree(%r)' % path)
            shutil.rmtree(path, onerror=on_error)
            failed = False
        except WindowsError:
            time.sleep(WAIT_TIME_SECS)
        if not failed: return

    msg('Warning: failed to remove directory %r' % path)


if __name__ == '__main__':
    main()
