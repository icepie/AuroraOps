#!/usr/bin/env bash

set -ex

cd ffmpeg
PKG_CONFIG_PATH="$DIST/lib/pkgconfig:/usr/lib64/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig:/usr/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}" ./configure \
	--prefix="$DIST" \
	--disable-debug \
	--enable-static \
	--disable-shared \
	--enable-pic \
	--enable-stripping \
	--disable-programs \
	--enable-gpl \
	--enable-libx264 \
	--disable-autodetect \
	--extra-cflags="$FFMPEG_CFLAGS" \
	--extra-ldflags="$FFMPEG_LIBRARY_PATH" \
	$FFMPEG_EXTRA_ARGS

make -j$NPROCS
make install
