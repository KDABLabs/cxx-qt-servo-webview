let
	pkgs = import <nixpkgs> {};
	servoSrc = pkgs.fetchFromGitHub {
		owner = "servo";
		repo = "servo";
		rev = "3a3e76a935f92ce84c24496cfc46207cd46088f6";
		hash = "sha256-iH81PTYjg7m4zuKIED14FiphP1ZKuB5CphOqBxkkgZc=";
	};
	servo = import (servoSrc.outPath + "/etc/shell.nix") {};
in
	servo.overrideAttrs (finalAttrs: previousAttrs: {
		buildInputs = previousAttrs.buildInputs ++ [
			pkgs.libGL
			pkgs.qt6.full
		];
	})
