# SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
# SPDX-FileContributor: Magnus Groß <magnus.gross@kdab.com>
#
# SPDX-License-Identifier: MPL-2.0

with import <nixpkgs> {};
let
	llvmPackages = llvmPackages_14;
	stdenv = stdenvAdapters.useMoldLinker clangStdenv;
in
	stdenv.mkDerivation {
		name = "cxx-qt-servo-webview";

		buildInputs = [
			fontconfig freetype libunwind
			xorg.libxcb
			xorg.libX11
			xorg.libXcursor
			xorg.libXrandr
			xorg.libXi
			xorg.xcbutilkeysyms
			xorg.libXinerama
			xcb-util-cursor
			libxkbcommon
			gst_all_1.gstreamer
			gst_all_1.gst-plugins-base
			gst_all_1.gst-plugins-bad
			rustup
			taplo
			llvmPackages.bintools
			udev
			cmake dbus gcc git pkg-config which llvm perl yasm m4
			gnumake
			libGL
			qt6.full
			stdenv.cc.cc.lib
			mold
			libclang
		];
		LD_LIBRARY_PATH = lib.makeLibraryPath [zlib xorg.libXcursor xorg.libXrandr xorg.libXi libxkbcommon vulkan-loader stdenv.cc.cc];

		shellHook = ''
			# see https://github.com/servo/mozjs/blob/20f7934762a6a1d4751353c8d024a0185ba85547/shell.nix#L11-L16
			export AS="$CC -c"
		'';
	}
