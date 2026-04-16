%global app_id io.github.kalaksi.Lightkeeper
%global licdir %{_defaultlicensedir}/%{name}

Name:           lightkeeper
Version:        0.38.2
Release:        1%{?dist}
Summary:        Customizable Linux server management tool over SSH

License:        GPL-3.0-or-later AND GPL-2.0-only AND MIT AND BSD-3-Clause AND LGPL-2.0-or-later
URL:            https://github.com/kalaksi/lightkeeper
Source0:        %{url}/archive/refs/tags/v%{version}.tar.gz#/%{name}-%{version}.tar.gz

ExclusiveArch:  x86_64

# Rust toolchain
BuildRequires:  rust >= 1.88
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  gcc-c++
BuildRequires:  make

# Qt 6
BuildRequires:  qt6-qtbase-devel
BuildRequires:  qt6-qtdeclarative-devel

# Native library headers
BuildRequires:  dbus-devel
BuildRequires:  liboping-devel

# Vendored OpenSSL build
BuildRequires:  perl-File-Compare
BuildRequires:  perl-File-Copy
BuildRequires:  perl-FindBin
BuildRequires:  perl-IPC-Cmd
BuildRequires:  perl-Time-Piece

# Runtime: Qt 6 QML modules
Requires:       qt6-qtbase
Requires:       qt6-qtdeclarative
Requires:       qt6-qtsvg
Requires:       qt6-qt5compat
Requires:       qt6-qtshadertools
Requires:       kf6-kirigami
Requires:       kf6-syntax-highlighting

# Runtime: native libraries
Requires:       liboping
Requires:       dbus-libs

# Optional but recommended for the integrated code editor
Recommends:     qt6-qtwebengine

%description
LightkeeperRM (Remote Management) is a customizable server management and
monitoring tool. It works as a drop-in replacement for maintaining servers
over SSH. Agentless: no additional daemons or agents needed on target hosts.

%prep
%autosetup -n %{name}-%{version}

%build
# Build qmltermwidget (bundled QML terminal plugin)
pushd third_party/qmltermwidget
qmake6
%make_build
popd

# Build lightkeeper binary.
# Set compile-time env vars for system-installed QML module paths.
LIGHTKEEPER_QML_LIB_DIR=%{_libdir}/lightkeeper/qml \
LIGHTKEEPER_QML_DATA_DIR=%{_datadir}/lightkeeper/qml \
cargo build --release

%install
# Binary
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Desktop entry
install -Dm644 flatpak/%{app_id}.desktop \
    %{buildroot}%{_datadir}/applications/%{app_id}.desktop

# AppStream metainfo
install -Dm644 flatpak/%{app_id}.metainfo.xml \
    %{buildroot}%{_metainfodir}/%{app_id}.metainfo.xml

# Icon
install -Dm644 flatpak/%{app_id}-rounded.png \
    %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/%{app_id}.png

# --- QML modules ---

# qmltermwidget (native plugin -> libdir)
tmpdir=$(mktemp -d)
%make_install -C third_party/qmltermwidget INSTALL_ROOT="$tmpdir"
mkdir -p %{buildroot}%{_libdir}/lightkeeper/qml
test -d "$tmpdir%{_prefix}/%{_lib}/qt6/qml/QMLTermWidget"
cp -a "$tmpdir%{_prefix}/%{_lib}/qt6/qml/QMLTermWidget" %{buildroot}%{_libdir}/lightkeeper/qml/
rm -rf "$tmpdir"

# Pure QML third-party modules (-> datadir)
mkdir -p %{buildroot}%{_datadir}/lightkeeper/qml

# ChartJs2QML
cp -a third_party/ChartJs2QML/ChartJs %{buildroot}%{_datadir}/lightkeeper/qml/

# Lighthouse components (FileBrowser, AceEditor, FilePermissionsDialog)
cp -a third_party/qml-lighthouse-components/Lighthouse %{buildroot}%{_datadir}/lightkeeper/qml/
find %{buildroot}%{_datadir}/lightkeeper/qml/Lighthouse/AceEditor/ace-builds \
    -mindepth 1 -maxdepth 1 ! -name LICENSE ! -name src-min-noconflict -exec rm -rf {} +

# Lightkeeper QML type info
cp -a src/frontend/qt/qml_types/Lightkeeper %{buildroot}%{_datadir}/lightkeeper/qml/

# License texts: paths under %{licdir} mirror the source tree (unique full paths)
install -Dpm644 LICENSE %{buildroot}%{licdir}/LICENSE
install -Dpm644 third_party/ChartJs2QML/LICENSE %{buildroot}%{licdir}/third_party/ChartJs2QML/LICENSE
install -Dpm644 \
    third_party/qml-lighthouse-components/Lighthouse/AceEditor/LICENSE \
    %{buildroot}%{licdir}/third_party/qml-lighthouse-components/Lighthouse/AceEditor/LICENSE
install -Dpm644 \
    third_party/qml-lighthouse-components/Lighthouse/AceEditor/ace-builds/LICENSE \
    %{buildroot}%{licdir}/third_party/qml-lighthouse-components/Lighthouse/AceEditor/ace-builds/LICENSE
install -Dpm644 \
    third_party/qml-lighthouse-components/Lighthouse/FileBrowser/LICENSE \
    %{buildroot}%{licdir}/third_party/qml-lighthouse-components/Lighthouse/FileBrowser/LICENSE
install -Dpm644 \
    third_party/qml-lighthouse-components/Lighthouse/FilePermissionsDialog/LICENSE \
    %{buildroot}%{licdir}/third_party/qml-lighthouse-components/Lighthouse/FilePermissionsDialog/LICENSE
install -Dpm644 third_party/qmltermwidget/LICENSE \
    %{buildroot}%{licdir}/third_party/qmltermwidget/LICENSE
install -Dpm644 third_party/qmltermwidget/LICENSE.BSD-3-clause \
    %{buildroot}%{licdir}/third_party/qmltermwidget/LICENSE.BSD-3-clause
install -Dpm644 third_party/qmltermwidget/LICENSE.LGPL2+ \
    %{buildroot}%{licdir}/third_party/qmltermwidget/LICENSE.LGPL2+

%files
%license %{licdir}/LICENSE
%license %{licdir}/third_party/ChartJs2QML/LICENSE
%license %{licdir}/third_party/qml-lighthouse-components/Lighthouse/AceEditor/LICENSE
%license %{licdir}/third_party/qml-lighthouse-components/Lighthouse/AceEditor/ace-builds/LICENSE
%license %{licdir}/third_party/qml-lighthouse-components/Lighthouse/FileBrowser/LICENSE
%license %{licdir}/third_party/qml-lighthouse-components/Lighthouse/FilePermissionsDialog/LICENSE
%license %{licdir}/third_party/qmltermwidget/LICENSE
%license %{licdir}/third_party/qmltermwidget/LICENSE.BSD-3-clause
%license %{licdir}/third_party/qmltermwidget/LICENSE.LGPL2+
%license NOTICES
%doc README.md
%{_bindir}/%{name}
%{_datadir}/applications/%{app_id}.desktop
%{_metainfodir}/%{app_id}.metainfo.xml
%{_datadir}/icons/hicolor/128x128/apps/%{app_id}.png
%{_libdir}/lightkeeper/
%{_datadir}/lightkeeper/

%changelog
* Wed Apr 15 2026 kalaksi <kalaksi@users.noreply.github.com> - 0.38.2-1
  - Initial Fedora package
