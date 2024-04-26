# SPDX-FileCopyrightText: 2024 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
# SPDX-FileContributor: Magnus Groß <magnus.gross@kdab.com>
#
# SPDX-License-Identifier: MPL-2.0

with import <nixpkgs> {};
with import (fetchTarball "https://github.com/nix-community/nixGL/archive/489d6b095ab9d289fe11af0219a9ff00fe87c7c5.tar.gz") { enable32bits = false; };
let
	pkgs_gnumake_4_3 = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/6adf48f53d819a7b6e15672817fa1e78e5f4e84f.tar.gz") {};
	llvmPackages = llvmPackages_14; # servo/servo#31059
	stdenv = stdenvAdapters.useMoldLinker llvmPackages.stdenv;
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
			llvmPackages.llvm
			llvmPackages.libclang
			udev
			cmake dbus gcc git pkg-config which llvm perl yasm m4
			pkgs_gnumake_4_3.gnumake # servo/mozjs#375
			libGL
			qt6.full
			stdenv.cc.cc.lib
			mold
		];
		LD_LIBRARY_PATH = lib.makeLibraryPath [zlib xorg.libXcursor xorg.libXrandr xorg.libXi libxkbcommon vulkan-loader stdenv.cc.cc];
		LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

		shellHook = ''
			# see https://github.com/servo/mozjs/blob/20f7934762a6a1d4751353c8d024a0185ba85547/shell.nix#L11-L16
			export AS="$CC -c"
		'';
	}
