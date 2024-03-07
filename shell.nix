let
	servo = import servo/etc/shell.nix {};
	pkgs = import <nixpkgs> {};
in
	servo.overrideAttrs (finalAttrs: previousAttrs: {
		buildInputs = previousAttrs.buildInputs ++ [
			pkgs.libGL
			pkgs.qt6.full
		];
	})
