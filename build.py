#!/usr/bin/env python3

TEXT_PATH = 'text'
OUT_PATH = 'target'
BOOK_OUT_PATH = OUT_PATH + '/book'
STATIC_PATH = 'static'
RUSTBOOK_OUT_PATH = '_book'
REDIRECTS = 'redirects.json'

PUBLISH_BRANCH = 'gh-pages'

TEMP_PREFIX = 'tlborm-build-'
WATCH_DELAY = 0.25 # sec
WATCH_SLEEP = 0.5 # sec

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager, ExitStack
from itertools import chain

TRACE = os.environ.get('TLBORM_TRACE_BUILD', '') != ''

REDIRECT_TEMPLATE = """<!DOCTYPE HTML>
<html>
<head>
    <meta charset="UTF-8">
    <title>The Little Book of Rust Macros</title>
    <link rel="stylesheet" type="text/css" href="%(rel)s/rust-book.css">
    <style type="text/css">

    body {
        position: absolute;
        top: 50%%;
        left: 50%%;
        margin-right: -50%%;
        transform: translate(-50%%, -50%%);
    }

    body > p:first-of-type {
        text-align: center;
    }

    </style>
    <meta http-equiv="refresh" content="0; url=%(dest)s">
</head>
<body>

<h1><a href="%(dest)s">Content Moved</a></h1>
<p><a href="%(dest)s">Follow the redirection</a> if it does not work automatically.</p>

</body>
</html>
"""

def main():
    args = sys.argv[1:]
    if args == []:
        args = ['build']

    def usage():
        print('Usage: build.py [build | open | publish | watch | help]')
        return 1
    
    if '--help' in args or 'help' in args or len(args) != 1:
        return usage()

    if args[0] == 'build':
        build()

    elif args[0] == 'open':
        do_open()

    elif args[0] == 'publish':
        publish()

    elif args[0] == 'watch':
        watch()

    else:
        return usage()

def build():
    msg('Building...')
    sh('rustbook', 'build', TEXT_PATH)

    if os.path.exists(OUT_PATH):
        really_rmtree(OUT_PATH)

    gen_redirs()

    copy_merge(src=RUSTBOOK_OUT_PATH, dst=BOOK_OUT_PATH)
    copy_merge(src=STATIC_PATH, dst=OUT_PATH)
    msg('.. done.')

def gen_redirs():
    msg_trace('.. generating redirects...')
    base_path = BOOK_OUT_PATH
    redirs = json.loads(open(REDIRECTS, 'rt').read())
    for entry in redirs:
        src = entry[0]
        dst = entry[1]
        rel = os.path.relpath(dst, os.path.dirname(src))
        rel = rel.replace("\\", "/")
        msg_trace('   .. %s -> %s / %s' % (src, dst, rel))

        rel_base = os.path.dirname(rel) or '.'

        page = REDIRECT_TEMPLATE % {'dest': rel+'.html', 'rel': rel_base}
        redir = os.path.join(base_path, src+'.html')
        redir_dir = os.path.dirname(redir)
        if not os.path.exists(redir_dir): os.makedirs(redir_dir)
        open(redir, 'wt').write(page)
    msg_trace('   .. done.')

def do_open():
    msg('Opening...')
    os.startfile(os.path.join(OUT_PATH, 'index.html'))
    msg('.. done.')

def publish():
    if sh_eval('git', 'symbolic-ref', '--short', 'HEAD') != 'master':
        raise 'Not publishing: not on master branch!'

    init_pub_branch()

    msg('Publishing...')

    repo_path = os.getcwd()
    msg_trace('.. repo_path = %r' % repo_path)

    last_rev = sh_eval('git', 'rev-parse', 'HEAD')
    last_msg = sh_eval('git', 'log', '-l', '--pretty=%B')
    msg_trace('.. last_rev = %r' % last_rev)
    msg_trace('.. last_msg = %r' % last_msg)

    with ExitStack() as stack:
        master_tmp = stack.enter_context(mkdtemp(prefix=TEMP_PREFIX))
        gh_pages_tmp = stack.enter_context(mkdtemp(prefix=TEMP_PREFIX))
        msg_trace('.. master_tmp = %r' % master_tmp)
        msg_trace('.. gh_pages_tmp = %r' % gh_pages_tmp)

        msg('.. cloning...')
        sh('git', 'clone', '-qb', 'master', repo_path, master_tmp)
        sh('git', 'clone', '-qb', PUBLISH_BRANCH, repo_path, gh_pages_tmp)

        with pushd(master_tmp):
            msg('.. running build...')
            build()

        msg('.. copying to %s...' % PUBLISH_BRANCH)
        copy_merge(
            src=os.path.join(master_tmp, OUT_PATH),
            dst=gh_pages_tmp)

        with pushd(gh_pages_tmp):
            msg('.. committing changes...')
            sh('git', 'add', '.')
            sh('git', 'commit',
               '-m', "Update docs for %s" % last_rev[:7])

            sh('git', 'push', '-qu', 'origin', PUBLISH_BRANCH)

    sh('git', 'push', '-q', 'origin', 'master', PUBLISH_BRANCH)
    msg('.. done.')

def watch():
    try:
        import watchdog
    except ImportError:
        msg('Cannot watch: could not import watchdog.')
        msg('Try installing with `pip install watchdog`.')
        return

    msg('Watching for changes...')

    from watchdog.events import FileSystemEventHandler
    from watchdog.observers import Observer

    class RebuildHandler(FileSystemEventHandler):
        def __init__(self, box):
            self.box = box

        def on_any_event(self, event):
            last_ts = self.box[0]
            now = time.time()
            if last_ts is None:
                self.box[0] = now + WATCH_DELAY

    rebuild_after = [None]
    handler = RebuildHandler(rebuild_after)

    observer = Observer()
    observer.schedule(handler, TEXT_PATH, recursive=True)
    observer.schedule(handler, STATIC_PATH, recursive=True)
    observer.start()

    def enable_tracing():
        global TRACE
        if not TRACE:
            TRACE = True
            msg_trace('Enabled tracing due to exception.')

    try:
        import traceback
        while True:
            ts = rebuild_after[0]

            if ts is not None and time.time() >= ts:
                rebuild_after[0] = None
                try:
                    build()
                except:
                    traceback.print_exc()
                    enable_tracing()

            time.sleep(WATCH_SLEEP)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()

    msg('.. done.')

def init_pub_branch():
    msg_trace('init_pub_branch()')

    branches = {b[2:].strip()
                for b
                in sh_eval('git', 'branch', dont_strip=True).splitlines()}
    msg_trace('.. branches = %r' % branches)
    if PUBLISH_BRANCH in branches:
        msg_trace('.. publish branch exists')
        return

    msg("Initialising %s branch..." % PUBLISH_BRANCH)

    repo_path = os.getcwd()
    msg_trace('.. repo_path = %r' % repo_path)

    with ExitStack() as stack:
        tmp = stack.enter_context(mkdtemp(prefix=TEMP_PREFIX))
        msg_trace('.. tmp = %r' % tmp)

        msg(".. cloning...")
        sh('git', 'init', '-q', tmp)
        with pushd(tmp):
            sh('git', 'checkout', '-q', '--orphan', PUBLISH_BRANCH)
            sh('git', 'commit', '-qm', "Initial commit.", '--allow-empty')
            sh('git', 'remote', 'add', 'origin', repo_path)
            sh('git', 'push', '-q', 'origin', PUBLISH_BRANCH)

    msg('.. done.')

def copy_merge(src, dst):
    msg_trace('copy_merge(%r, %r)' % (src, dst))
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
        subprocess.check_call(cmd, shell=(os.name == 'nt'))
    except:
        msg_trace('FAILED!')
        raise

def sh_eval(*cmd, codec='utf-8', dont_strip=False):
    msg_trace('=%s' % ' '.join(escape_argument(str(a)) for a in cmd))
    result = None
    try:
        result = subprocess.check_output(cmd, shell=True).decode(codec)
        if not dont_strip:
            result = result.strip()
    except:
        msg_trace('FAILED!')
        raise
    return result

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
    msg_trace('really_rmtree(%r)' % path)
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

@contextmanager
def pushd(path):
    msg_trace('pushd(%r)' % path)
    old_path = os.getcwd()
    os.chdir(path)
    try:
        yield
    finally:
        msg_trace('popd(%r)' % old_path)
        os.chdir(old_path)

@contextmanager
def mkdtemp(prefix=None):
    path = tempfile.mkdtemp(prefix=prefix)
    msg_trace('mkdtemp(..) = %r' % path)
    try:
        yield path
    finally:
        really_rmtree(path)

if __name__ == '__main__':
    sys.exit(main() or 0)
