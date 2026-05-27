#!/usr/bin/env bash

set -ex

test -d x264 || git clone --depth 1 -b stable https://code.videolan.org/videolan/x264.git x264
test -d ffmpeg || git clone --depth 1 -b n8.0 https://git.ffmpeg.org/ffmpeg.git ffmpeg
if [ "$TARGET_OS" == "linux" ]; then
    test -d nv-codec-headers || git clone --depth 1 https://git.videolan.org/git/ffmpeg/nv-codec-headers.git
    test -d libva || git clone --depth 1 -b 2.22.0 https://github.com/intel/libva
fi
if [ "$TARGET_OS" == "windows" ]; then
    test -d nv-codec-headers || git clone --depth 1 https://git.videolan.org/git/ffmpeg/nv-codec-headers.git
fi

if [ "$TARGET_OS" == "windows" ] && [ "$HOST_OS" == "windows" ]; then
    cd ffmpeg
    git apply ../command_limit.patch || true
    git apply ../awk.patch || {
        cat > msvc_dep.awk <<'EOF'
/including/ { sub(/^.*file: */, ""); gsub(/\\/, "/"); if (!match($0, / /)) print target ":", $0 }
EOF
        python3 - <<'PY'
from pathlib import Path

path = Path("configure")
text = path.read_text()
old = r"""_DEPCMD='$(DEP$(1)) $(DEP$(1)FLAGS) $($(1)DEP_FLAGS) $< 2>&1 | awk '\''/including/ { sub(/^.*file: */, ""); gsub(/\\/, "/"); if (!match($$0, / /)) print "$@:", $$0 }'\'' > $(@:.o=.d)'"""
new = r"""_DEPCMD='$(DEP$(1)) $(DEP$(1)FLAGS) $($(1)DEP_FLAGS) $< 2>&1 | awk -v target="$@" -f ./msvc_dep.awk > $(@:.o=.d)'"""
if old in text:
    path.write_text(text.replace(old, new, 1))
elif new not in text:
    raise SystemExit("Unable to patch FFmpeg MSVC dependency command")
PY
    }
fi
