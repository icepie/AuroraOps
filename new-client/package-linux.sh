#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="${DIST_DIR:-${ROOT_DIR}/dist}"
WORK_DIR="${WORK_DIR:-${ROOT_DIR}/target/package-linux}"
PACKAGE_NAME="auroraops-agent"
VERSION="${VERSION:-$(awk -F '"' '/^version = / { print $2; exit }' "${ROOT_DIR}/Cargo.toml")}"
RELEASE="${RELEASE:-1}"
BUILD_MODE="${BUILD_MODE:-release}"
FEATURES="${FEATURES-ffmpeg-system}"
DEB_DEPENDS_BASE="libc6, libgcc-s1 | libgcc1, systemd, curl, xdg-utils, python3, policykit-1 | polkitd, libx11-6, libxext6, libxrandr2, libxfixes3, libxcomposite1, libxi6, libxtst6, libxinerama1, libxcursor1, libxkbcommon0, libwayland-client0, libwayland-cursor0, libdbus-1-3, libssl3 | libssl1.1, libglib2.0-0, libgstreamer1.0-0, libgstreamer-plugins-base1.0-0, libpango-1.0-0, libcairo2, libpangocairo-1.0-0"
DEB_DEPENDS_FFMPEG="libavformat62 | libavformat61 | libavformat60 | libavformat59 | libavformat58, libavfilter11 | libavfilter10 | libavfilter9 | libavfilter8 | libavfilter7, libavcodec62 | libavcodec61 | libavcodec60 | libavcodec59 | libavcodec58, libavutil60 | libavutil59 | libavutil58 | libavutil57 | libavutil56, libswscale9 | libswscale8 | libswscale7 | libswscale6 | libswscale5, libswresample6 | libswresample5 | libswresample4 | libswresample3"
DEB_RECOMMENDS="gstreamer1.0-plugins-base, gstreamer1.0-pipewire, libuinput-tools, whiptail | dialog, firefox | chromium | chromium-browser"
RPM_REQUIRES_BASE=(
  "systemd"
  "gstreamer1"
  "gstreamer1-plugins-base"
  "glib2"
  "dbus-libs"
  "openssl-libs"
  "pango"
  "cairo"
  "libX11"
  "libXext"
  "libXrandr"
  "libXfixes"
  "libXcomposite"
  "libXi"
  "libXtst"
  "libXinerama"
  "libXcursor"
  "libxkbcommon"
  "wayland"
  "curl"
  "xdg-utils"
  "python3"
  "newt"
  "polkit"
)
RPM_REQUIRES_FFMPEG=(
  "ffmpeg-libs"
)

usage() {
  cat <<'EOF'
Usage: ./package-linux.sh [--skip-build] [--deb-only] [--rpm-only]

Environment:
  VERSION       Package version. Defaults to Cargo.toml package version.
  RELEASE       Package release number. Defaults to 1.
  DIST_DIR      Output directory. Defaults to ./dist.
  FEATURES      Cargo features. Defaults to ffmpeg-system.
  BUILD_MODE    Cargo profile. Defaults to release.
EOF
}

SKIP_BUILD=0
BUILD_DEB=1
BUILD_RPM=1
while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-build)
      SKIP_BUILD=1
      ;;
    --deb-only)
      BUILD_RPM=0
      ;;
    --rpm-only)
      BUILD_DEB=0
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if [ -z "${VERSION}" ]; then
  echo "Unable to determine package version." >&2
  exit 1
fi

case ",${FEATURES}," in
  *,ffmpeg-system,*)
  DEB_DEPENDS="${DEB_DEPENDS_BASE}, ${DEB_DEPENDS_FFMPEG}"
  RPM_REQUIRES=("${RPM_REQUIRES_BASE[@]}" "${RPM_REQUIRES_FFMPEG[@]}")
  ;;
  *)
  DEB_DEPENDS="${DEB_DEPENDS_BASE}"
  RPM_REQUIRES=("${RPM_REQUIRES_BASE[@]}")
  ;;
esac

case "$(uname -m)" in
  x86_64|amd64)
    DEB_ARCH="amd64"
    RPM_ARCH="x86_64"
    ;;
  aarch64|arm64)
    DEB_ARCH="arm64"
    RPM_ARCH="aarch64"
    ;;
  *)
    DEB_ARCH="$(uname -m)"
    RPM_ARCH="$(uname -m)"
    ;;
esac

PROFILE_DIR="debug"
if [ "${BUILD_MODE}" = "release" ]; then
  PROFILE_DIR="release"
fi

if [ "${SKIP_BUILD}" -eq 0 ]; then
  CARGO_ARGS=(build "--bin" "${PACKAGE_NAME}")
  if [ "${BUILD_MODE}" = "release" ]; then
    CARGO_ARGS+=(--release)
  fi
  if [ -n "${FEATURES}" ]; then
    CARGO_ARGS+=(--features "${FEATURES}")
  fi
  (cd "${ROOT_DIR}" && cargo "${CARGO_ARGS[@]}")
fi

BIN_PATH="${ROOT_DIR}/target/${PROFILE_DIR}/${PACKAGE_NAME}"
if [ ! -x "${BIN_PATH}" ]; then
  echo "Built binary not found: ${BIN_PATH}" >&2
  exit 1
fi
rm -rf "${WORK_DIR}"
mkdir -p "${DIST_DIR}" "${WORK_DIR}"

install_common_tree() {
  local stage="$1"
  install -d "${stage}/opt/auroraops"
  install -d "${stage}/etc/auroraops"
  install -d "${stage}/etc/systemd/system"
  install -d "${stage}/usr/share/applications"
  install -d "${stage}/usr/share/doc/${PACKAGE_NAME}"
  install -m 0755 "${BIN_PATH}" "${stage}/opt/auroraops/${PACKAGE_NAME}"
  install -m 0755 "${ROOT_DIR}/auroraops-client-launcher" "${stage}/opt/auroraops/auroraops-client-launcher"
  install -m 0755 "${ROOT_DIR}/auroraops-client-config" "${stage}/opt/auroraops/auroraops-client-config"
  install -m 0755 "${ROOT_DIR}/auroraops-uinput-setup" "${stage}/opt/auroraops/auroraops-uinput-setup"
  install -m 0644 "${ROOT_DIR}/auroraops-agent.service" "${stage}/etc/systemd/system/auroraops-agent.service"
  install -m 0644 "${ROOT_DIR}/packaging/agent-config.json" "${stage}/etc/auroraops/agent-config.json"
  install -m 0644 "${ROOT_DIR}/auroraops-agent.desktop" "${stage}/usr/share/applications/auroraops-agent.desktop"
  install -m 0644 "${ROOT_DIR}/Readme.md" "${stage}/usr/share/doc/${PACKAGE_NAME}/README.md"
}

build_deb() {
  command -v dpkg-deb >/dev/null 2>&1 || {
    echo "dpkg-deb is required for deb packaging." >&2
    exit 1
  }
  local stage="${WORK_DIR}/deb/${PACKAGE_NAME}_${VERSION}-${RELEASE}_${DEB_ARCH}"
  install_common_tree "${stage}"
  install -d "${stage}/DEBIAN"
  cat > "${stage}/DEBIAN/control" <<EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}-${RELEASE}
Section: utils
Priority: optional
Architecture: ${DEB_ARCH}
Maintainer: AuroraOps <opensource@auroraops.local>
Depends: ${DEB_DEPENDS}
Recommends: ${DEB_RECOMMENDS}
Description: AuroraOps remote desktop and device agent
 AuroraOps Agent runs Weylus headlessly, keeps a systemd service online,
 registers this host to AuroraOps Server, and bridges terminal and remote
 desktop control traffic over the AuroraOps TCP channel.
EOF
  cat > "${stage}/DEBIAN/conffiles" <<'EOF'
/etc/auroraops/agent-config.json
EOF
  install -m 0755 "${ROOT_DIR}/packaging/deb/postinst" "${stage}/DEBIAN/postinst"
  install -m 0755 "${ROOT_DIR}/packaging/deb/prerm" "${stage}/DEBIAN/prerm"
  install -m 0755 "${ROOT_DIR}/packaging/deb/postrm" "${stage}/DEBIAN/postrm"
  dpkg-deb --build --root-owner-group "${stage}" "${DIST_DIR}/${PACKAGE_NAME}_${VERSION}-${RELEASE}_${DEB_ARCH}.deb"
}

build_rpm() {
  command -v rpmbuild >/dev/null 2>&1 || {
    echo "rpmbuild is required for rpm packaging." >&2
    exit 1
  }
  local rpmbuild_dir="${WORK_DIR}/rpmbuild"
  local stage="${WORK_DIR}/rpm/stage"
  install_common_tree "${stage}"
  install -d "${rpmbuild_dir}/BUILD" "${rpmbuild_dir}/BUILDROOT" "${rpmbuild_dir}/RPMS" "${rpmbuild_dir}/SOURCES" "${rpmbuild_dir}/SPECS" "${rpmbuild_dir}/SRPMS"
  tar -C "${stage}" -czf "${rpmbuild_dir}/SOURCES/${PACKAGE_NAME}-${VERSION}.tar.gz" .
  cat > "${rpmbuild_dir}/SPECS/${PACKAGE_NAME}.spec" <<EOF
Name: ${PACKAGE_NAME}
Version: ${VERSION}
Release: ${RELEASE}%{?dist}
Summary: AuroraOps remote desktop and device agent
License: AGPL-3.0-or-later
URL: https://github.com/icepie/AuroraOps
BuildArch: ${RPM_ARCH}
Source0: %{name}-%{version}.tar.gz
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd
$(printf 'Requires: %s\n' "${RPM_REQUIRES[@]}")

%description
AuroraOps Agent runs Weylus headlessly, keeps a systemd service online,
registers this host to AuroraOps Server, and bridges terminal and remote
desktop control traffic over the AuroraOps TCP channel.

%prep

%build

%install
mkdir -p %{buildroot}
tar -C %{buildroot} -xzf %{SOURCE0}

%post
$(sed 's/^/  /' "${ROOT_DIR}/packaging/rpm/postinstall.sh")

%preun
$(sed 's/^/  /' "${ROOT_DIR}/packaging/rpm/preuninstall.sh")

%postun
$(sed 's/^/  /' "${ROOT_DIR}/packaging/rpm/postuninstall.sh")

%files
%license /usr/share/doc/${PACKAGE_NAME}/README.md
/opt/auroraops/${PACKAGE_NAME}
/opt/auroraops/auroraops-client-launcher
/opt/auroraops/auroraops-client-config
/opt/auroraops/auroraops-uinput-setup
/etc/systemd/system/auroraops-agent.service
/usr/share/applications/auroraops-agent.desktop
%config(noreplace) /etc/auroraops/agent-config.json

%changelog
* Sat May 23 2026 AuroraOps <opensource@auroraops.local> - ${VERSION}-${RELEASE}
- Package AuroraOps Agent as a systemd service.
EOF
  rpmbuild --define "_topdir ${rpmbuild_dir}" -bb "${rpmbuild_dir}/SPECS/${PACKAGE_NAME}.spec"
  find "${rpmbuild_dir}/RPMS" -type f -name '*.rpm' -exec cp {} "${DIST_DIR}/" \;
}

if [ "${BUILD_DEB}" -eq 1 ]; then
  build_deb
fi
if [ "${BUILD_RPM}" -eq 1 ]; then
  build_rpm
fi

echo "Package artifacts:"
find "${DIST_DIR}" -maxdepth 1 -type f \( -name '*.deb' -o -name '*.rpm' \) -printf '  %p\n' | sort
