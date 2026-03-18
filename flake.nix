{
  description = "scientisto environment";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/master";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    crane.url = "github:ipetkov/crane/v0.23.0";
    nix-lib.url = "github:hoprnet/nix-lib";
    # pin it to a version which we are compatible with
    pre-commit.url = "github:cachix/git-hooks.nix";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-root.url = "github:srid/flake-root";

    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    nix-lib.inputs.nixpkgs.follows = "nixpkgs";
    nix-lib.inputs.flake-utils.follows = "flake-utils";
    nix-lib.inputs.crane.follows = "crane";
    nix-lib.inputs.flake-parts.follows = "flake-parts";
    nix-lib.inputs.rust-overlay.follows = "rust-overlay";
    nix-lib.inputs.treefmt-nix.follows = "treefmt-nix";
    nix-lib.inputs.nixpkgs-unstable.follows = "nixpkgs-unstable";
    pre-commit.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      nixpkgs-unstable,
      flake-utils,
      flake-parts,
      rust-overlay,
      crane,
      nix-lib,
      pre-commit,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.flake-root.flakeModule
      ];
      perSystem =
        {
          config,
          lib,
          system,
          ...
        }:
        let
          rev = toString (self.shortRev or self.dirtyShortRev);
          fs = lib.fileset;
          localSystem = system;
          overlays = [
            (import rust-overlay)
          ];
          pkgs = import nixpkgs { inherit localSystem overlays; };
          pkgs-unstable = import nixpkgs-unstable { inherit localSystem overlays; };
          buildPlatform = pkgs.stdenv.buildPlatform;

          # Import nix-lib for shared Nix utilities
          nixLib = nix-lib.lib.${system};

          # Wrapper for rustfmt to fix macOS dylib loading issue
          # On macOS, rust-overlay symlinks rustfmt to a standalone package that can't find its dylibs.
          # This wrapper sets DYLD_LIBRARY_PATH to the toolchain's lib directory.
          nightlyToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          rustfmtWrapper = pkgs.writeShellScriptBin "rustfmt" ''
            export DYLD_LIBRARY_PATH="${nightlyToolchain}/lib:$DYLD_LIBRARY_PATH"
            exec "${nightlyToolchain}/bin/rustfmt" "$@"
          '';

          craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);
          crateInfoOriginal = craneLib.crateNameFromCargoToml {
            cargoToml = ./Cargo.toml;
          };
          crateInfo = {
            pname = "scientisto";
            # normalize the version to not include any suffixes so the cache
            # does not get busted
            version = pkgs.lib.strings.concatStringsSep "." (
              pkgs.lib.lists.take 3 (builtins.splitVersion crateInfoOriginal.version)
            );
          };

          # Use nix-lib's source filtering for better rebuild performance
          depsSrc = nixLib.mkDepsSrc {
            root = ./.;
            inherit fs;
          };
          src = nixLib.mkSrc {
            root = ./.;
            inherit fs;
            extraFiles = [ ];
          };
          testSrc = nixLib.mkTestSrc {
            root = ./.;
            inherit fs;
            extraFiles = [
              ./tests
              (fs.fileFilter (file: file.hasExt "snap") ./.)
            ];
          };

          # Use nix-lib to create all rust builders for cross-compilation
          builders = nixLib.mkRustBuilders {
            inherit localSystem;
            rustToolchainFile = ./rust-toolchain.toml;
          };

          # Convenience aliases for builders
          rust-builder-local = builders.local;
          rust-builder-x86_64-linux = builders.x86_64-linux;
          rust-builder-x86_64-darwin = builders.x86_64-darwin;
          rust-builder-aarch64-linux = builders.aarch64-linux;
          rust-builder-aarch64-darwin = builders.aarch64-darwin;

          # Coverage builder with llvm-tools for code coverage instrumentation
          rust-builder-local-coverage = builders.localCoverage;

          # Nightly builder for docs and specific features
          rust-builder-local-nightly = nixLib.mkRustBuilder {
            inherit localSystem;
            rustToolchainFile = ./rust-toolchain.toml;
            useRustNightly = true;
          };

          scientistoBuildArgs = {
            inherit src depsSrc rev;
            cargoExtraArgs = "";
            cargoToml = ./Cargo.toml;
          };

          scientisto = rust-builder-local.callPackage nixLib.mkRustPackage scientistoBuildArgs;

          clippy = rust-builder-local.callPackage nixLib.mkRustPackage (
            scientistoBuildArgs // { runClippy = true; }
          );

          docs = rust-builder-local-nightly.callPackage nixLib.mkRustPackage (
            scientistoBuildArgs // { buildDocs = true; }
          );

          # Code coverage (outputs LCOV report)
          coverage = rust-builder-local-coverage.callPackage nixLib.mkRustPackage (
            scientistoBuildArgs
            // {
              src = testSrc;
              runCoverage = true;
              prependPackageName = false;
              cargoLlvmCovExtraArgs = "--lcov --output-path $out --lib";
            }
          );

          pre-commit-check = pre-commit.lib.${system}.run {
            src = ./.;
            hooks = {
              # https://github.com/cachix/git-hooks.nix
              treefmt.enable = false;
              treefmt.package = config.treefmt.build.wrapper;
              check-executables-have-shebangs.enable = true;
              check-shebang-scripts-are-executable.enable = true;
              check-case-conflicts.enable = true;
              check-symlinks.enable = true;
              check-merge-conflicts.enable = true;
              check-added-large-files.enable = true;
              commitizen.enable = true;
            };
            tools = pkgs;
            excludes = [
              ".gcloudignore"
            ];
          };

          # Development shells using nix-lib
          devShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "'scientisto' Development";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              pkgs-unstable.cargo-audit
              cargo-machete
              cargo-shear
              cargo-insta
            ];
            shellHook = ''
              ${pre-commit-check.shellHook}
            '';
          };

          # Development shell with Rust nightly
          devShellNightly = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "'scientisto' Development (Nightly)";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              pkgs-unstable.cargo-audit
              cargo-machete
              cargo-shear
              cargo-insta
            ];
            shellHook = ''
              # Fix macOS dylib loading for nightly rustfmt (rust-overlay symlink issue)
              export DYLD_LIBRARY_PATH="${nightlyToolchain}/lib:$DYLD_LIBRARY_PATH"
              ${pre-commit-check.shellHook}
            '';
            rustToolchain = nightlyToolchain;
          };

          ciShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "'scientisto' CI";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              act
              gh
              pkgs-unstable.cargo-audit
              cargo-machete
              cargo-shear
              graphviz
              zizmor
              gnupg
              perl
            ];
          };

          docsShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "'scientisto' Documentation";
            treefmtWrapper = config.treefmt.build.wrapper;
            treefmtPrograms = pkgs.lib.attrValues config.treefmt.build.programs;
            extraPackages = with pkgs; [
              html-tidy
              pandoc
              cargo-machete
              cargo-shear
            ];
            shellHook = ''
              ${pre-commit-check.shellHook}
            '';
            rustToolchain = nightlyToolchain;
          };

          coverageShell = nixLib.mkDevShell {
            rustToolchainFile = ./rust-toolchain.toml;
            shellName = "'scientisto' Coverage";
            withLlvmTools = true;
          };

          run-audit = flake-utils.lib.mkApp {
            drv = pkgs.writeShellApplication {
              name = "audit";
              runtimeInputs = [
                pkgs.cargo
                pkgs-unstable.cargo-audit
              ];
              text = ''
                cargo audit
              '';
            };
          };
        in
        {
          treefmt = {
            inherit (config.flake-root) projectRootFile;

            settings.global.excludes = [
              "**/*.id"
              "**/.cargo-ok"
              "**/.gitignore"
              ".actrc"
              ".dockerignore"
              ".editorconfig"
              ".gcloudignore"
              ".gitattributes"
              ".yamlfmt"
              "LICENSE"
              "Makefile"
              "docs/*"
              "*/snapshots/*"
              "target/*"
            ];

            programs.shfmt.enable = true;
            settings.formatter.shfmt.includes = [
              "*.sh"
            ];

            programs.yamlfmt.enable = true;
            settings.formatter.yamlfmt.includes = [
              ".github/labeler.yml"
              ".github/workflows/*.yaml"
            ];
            # trying setting from https://github.com/google/yamlfmt/blob/main/docs/config-file.md
            settings.formatter.yamlfmt.settings = {
              formatter.type = "basic";
              formatter.max_line_length = 120;
              formatter.trim_trailing_whitespace = true;
              formatter.scan_folded_as_literal = true;
              formatter.include_document_start = true;
            };

            programs.prettier.enable = true;
            settings.formatter.prettier.includes = [
              "*.md"
              "*.json"
            ];
            settings.formatter.prettier.excludes = [
              "*.yml"
              "*.yaml"
            ];
            programs.rustfmt.enable = true;
            # using the official Nixpkgs formatting
            # see https://github.com/NixOS/rfcs/blob/master/rfcs/0166-nix-formatting.md
            programs.nixfmt.enable = true;
            programs.taplo.enable = true;
            programs.ruff-format.enable = true;

            settings.formatter.rustfmt = {
              command = "${rustfmtWrapper}/bin/rustfmt";
            };
          };

          checks = {
            inherit clippy;
          };

          apps = {
            audit = run-audit;
          };

          packages = {
            inherit
              scientisto
              ;
            inherit docs;
            inherit coverage;
            inherit pre-commit-check;
            # binary packages
            default = scientisto;
          };

          devShells.default = devShell;
          devShells.nightly = devShellNightly;
          devShells.ci = ciShell;
          devShells.docs = docsShell;
          devShells.coverage = coverageShell;

          formatter = config.treefmt.build.wrapper;
        };
      # platforms which are supported as build environments
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
    };
}
