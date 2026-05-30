#!/usr/bin/env bash

set -ex

export TARGET_OS="$CARGO_CFG_TARGET_OS"

if [ "$OSTYPE" == "linux-gnu" ]; then
    export HOST_OS="linux"
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
    export HOST_OS="macos"
fi

if [ "$OS" == "Windows_NT" ]; then
    export HOST_OS="windows"
fi

[ -z "$DIST" ] && export DIST="$PWD/dist"
[ -z "$TARGET_OS" ] && export TARGET_OS="$HOST_OS"
[ -z "$TARGET_ARCH" ] && export TARGET_ARCH="$CARGO_CFG_TARGET_ARCH"
[ -z "$TARGET_ENV" ] && export TARGET_ENV="$CARGO_CFG_TARGET_ENV"
[ -z "$ENABLE_VAAPI" ] && export ENABLE_VAAPI="y"
[ -z "$ENABLE_VULKAN_VIDEO" ] && export ENABLE_VULKAN_VIDEO="n"

export NPROCS="$(nproc || echo 4)"

./download.sh

if [ "$TARGET_OS" == "windows" ]; then
    if [ "$HOST_OS" == "linux" ]; then
        export CROSS_COMPILE="x86_64-w64-mingw32-"
        export FFMPEG_EXTRA_ARGS="--arch=x86_64 --target-os=mingw64 \
            --cross-prefix=x86_64-w64-mingw32- --enable-nvenc --enable-ffnvcodec \
            --enable-mediafoundation --pkg-config=pkg-config --enable-d3d11va"
        export FFMPEG_CFLAGS="-I$DIST/include"
        export FFMPEG_LIBRARY_PATH="-L$DIST/lib"
    elif [ "$TARGET_ENV" != "msvc" ]; then
        export CC="clang"
        export CXX="clang++"
        export LD="clang"
        export AR="llvm-ar"
        export RANLIB="llvm-ranlib"
        export NM="llvm-nm"
        export STRIP="llvm-strip"
        export FFMPEG_LLVM_ARGS="--cc=clang --cxx=clang++ --ld=clang \
            --ar=llvm-ar --ranlib=llvm-ranlib --nm=llvm-nm --strip=llvm-strip"
        if [ "$TARGET_ARCH" == "aarch64" ]; then
            export FFMPEG_EXTRA_ARGS="--arch=aarch64 --target-os=win64 \
                --disable-asm --disable-nvenc --disable-ffnvcodec \
                --enable-mediafoundation --enable-d3d11va $FFMPEG_LLVM_ARGS"
            export X264_EXTRA_ARGS="--host=aarch64-w64-mingw32 --disable-asm"
        else
            export FFMPEG_EXTRA_ARGS="--arch=x86_64 --target-os=mingw64 \
                --enable-nvenc --enable-ffnvcodec \
                --enable-mediafoundation --pkg-config=pkg-config --enable-d3d11va \
                $FFMPEG_LLVM_ARGS"
        fi
        export FFMPEG_CFLAGS="-I$DIST/include"
        export FFMPEG_LIBRARY_PATH="-L$DIST/lib"
    elif [ "$TARGET_ARCH" == "aarch64" ]; then
        export CC="cl"
        export FFMPEG_EXTRA_ARGS="--toolchain=msvc --arch=aarch64 --target-os=win64 \
            --enable-cross-compile --disable-asm --disable-nvenc --disable-ffnvcodec \
            --enable-mediafoundation --enable-d3d11va"
        export FFMPEG_CFLAGS="-I$DIST/include"
        export FFMPEG_LIBRARY_PATH="-LIBPATH:$DIST/lib"
        export X264_EXTRA_ARGS="--host=aarch64-w64-mingw32 --disable-asm"
    else
        export CC="cl"
        export FFMPEG_EXTRA_ARGS="--toolchain=msvc --enable-nvenc --enable-ffnvcodec \
            --enable-mediafoundation --enable-d3d11va"
        export FFMPEG_CFLAGS="-I$DIST/include"
        export FFMPEG_LIBRARY_PATH="-LIBPATH:$DIST/lib"
    fi
else
    export FFMPEG_CFLAGS="-I$DIST/include"
    export FFMPEG_LIBRARY_PATH="-L$DIST/lib"
    if [ "$TARGET_OS" == "linux" ]; then
        export FFMPEG_EXTRA_ARGS="--enable-nvenc \
            --enable-ffnvcodec"
        if [ "$ENABLE_VULKAN_VIDEO" == "y" ]; then
            export FFMPEG_EXTRA_ARGS="$FFMPEG_EXTRA_ARGS \
                --enable-vulkan"
        fi
        if [ "$ENABLE_VAAPI" == "y" ]; then
            export FFMPEG_EXTRA_ARGS="$FFMPEG_EXTRA_ARGS \
                --enable-vaapi \
                --enable-libdrm \
                --enable-xlib"
        else
            export FFMPEG_EXTRA_ARGS="$FFMPEG_EXTRA_ARGS \
                --disable-vaapi"
        fi
    fi
    if [ "$TARGET_OS" == "macos" ]; then
        export FFMPEG_EXTRA_ARGS="--enable-videotoolbox"
    fi
fi

if [ "$ENABLE_LIBNPP" == "y" ]; then
    export FFMPEG_EXTRA_ARGS="$FFMPEG_EXTRA_ARGS --enable-libnpp --enable-nonfree"
fi

if [ "$TARGET_OS" == "windows" ] && [ "$HOST_OS" == "linux" ]; then
    export X264_EXTRA_ARGS="--cross-prefix=x86_64-w64-mingw32- --host=x86_64-w64-mingw32"
fi
./x264.sh
if [ "$TARGET_OS" == "windows" ] && [ "$HOST_OS" == "windows" ]; then
    if [ -f "$DIST/lib/libx264.lib" ]; then
        cp "$DIST/lib/libx264.lib" "$DIST/lib/x264.lib"
    fi
fi
if [ "$TARGET_OS" == "linux" ]; then
    ./nv-codec-headers.sh
    if [ "$ENABLE_VAAPI" == "y" ]; then
        ./libva.sh
    fi
fi
if [ "$TARGET_OS" == "windows" ]; then
    ./nv-codec-headers.sh
fi
./ffmpeg.sh

if [ "$TARGET_OS" == "windows" ] && [ "$HOST_OS" == "windows" ]; then
    cd "$DIST/lib"
    for l in *.a; do
        [ -e "$l" ] || continue
        d=${l#lib}
        cp "$l" "${d%.a}.lib"
    done
    if [ -f libx264.lib ]; then
        cp libx264.lib x264.lib
    fi
fi
