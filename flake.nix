{
  description = "Moji development environment";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      fenix,
      git-hooks,
      devshell,
    }:
    let
      # Everything that Nix supports right now
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      eachSystem =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f {
            inherit system;
            pkgs = nixpkgs.legacyPackages.${system};
            fenix-pkg = fenix.packages.${system};
          }
        );
    in
    {
      checks = eachSystem (
        {
          pkgs,
          system,
          fenix-pkg,
          ...
        }:
        let
          # Nightly enables a lot of nice things, but mainly it allows us to build with rustfmt
          rust-nightly = fenix-pkg.complete.withComponents [
            "cargo"
            "clippy"
            "rustc"
            "rustfmt"
            "rustc-codegen-cranelift-preview"
          ];
        in
        {
          pre-commit-check = git-hooks.lib.${system}.run {
            src = ./.;
            package = pkgs.prek; # Prek for parellelizism
            default_stages = [ "pre-push" ];
            hooks = {
              # We have convco here to enforce commit messages
              # The goal here is to NOT slow down dev productivity
              # So we're encouraging a workflow where, if you're in "flow"
              # Just don't worry about pushing. The annoying part can hit then
              convco = {
                enable = true;
                pass_filenames = false;
                entry = toString (
                  pkgs.writeShellScript "convco-pre-push" ''
                    while read local_ref local_sha remote_ref remote_sha; do
                      ${pkgs.convco}/bin/convco check "$remote_sha..$local_sha"
                    done
                  ''
                );
                stages = [ "pre-push" ];
              };

              # Formatting is PURELY for QOL
              # Consistency is key for building patterns, and to that end, a priority should be enabling reliable dev setups so this doesn't trip up on pre-push
              nixfmt.enable = true;

              rustfmt = {
                enable = true;
                packageOverrides.cargo = rust-nightly;
                packageOverrides.rustfmt = rust-nightly;
              };

              markdownfmt = {
                enable = true;
                name = "hongdown";
                entry = "hongdown --write";
                files = "\\.md$";
                language = "system";
              };

              # The main branch needs to always be green
              # Both passing all tests and avoiding any clippy lints
              testrust = {
                enable = true;
                name = "testrust";
                entry = "cargo nextest run";
                language = "system";
                pass_filenames = false;
                stages = [ "pre-merge-commit" ];
              };

              clippy = {
                enable = true;
                packageOverrides.cargo = rust-nightly;
                packageOverrides.clippy = rust-nightly;
                stages = [
                  "pre-merge-commit"
                  "pre-push"
                ];
              };
            };
          };
        }
      );

      devShells = eachSystem (
        {
          pkgs,
          system,
          fenix-pkg,
        }:
        let
          rust-nightly = fenix-pkg.complete.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rust-docs"
            "rustc"
            "rustfmt"
            "rustc-codegen-cranelift-preview"
          ];
          hooks = self.checks.${system}.pre-commit-check;

          # Sourcing from nushell for our commands
          mkCommand = name: help: category: {
            inherit name help category;
            command = "cd $PRJ_ROOT && nu .config/scripts/${name}.nu \"$@\"";
          };
        in
        {
          default = (devshell.legacyPackages.${system}.mkShell) {
            name = "NuNuShell";
            env = [
              {
                # TODO: See if there's a more reliable way to avoid this, just a linker issue I started facing
                name = "LIBRARY_PATH";
                value = "$(nix eval --raw nixpkgs#libiconv.outPath)/lib";
              }
              {
                name = "MAIN_PACKAGE";
                value = "nudox";
              }
              {
                name = "OUTPUT_DIRECTORY";
                value = "dist";
              }
            ];

            motd = ''
              $($(type -p kittysay) --think "the nu is the now" | dotacat)
            '';

            packages = [
              rust-nightly # Rust nightly toolchain
              pkgs.git # Version control
              pkgs.cargo-bump # Bump crate versions
              pkgs.rust-analyzer # Rust LSP server
              pkgs.flock # For managing shell concurrency
              pkgs.nixfmt # Nix formatter
              pkgs.tombi # TOML formatter/linter
              pkgs.typos # Source code spell checker
              pkgs.hongdown # Markdown formatting
              pkgs.kittysay # Cat ASCII art
              pkgs.marksman # Markdown LSP server
              pkgs.taplo # TOML LSP/formatter
              pkgs.cargo-nextest # Next-gen test runner
              pkgs.libiconv # Character encoding library, associated with linker error
              pkgs.nil # Nix LSP server
              pkgs.jsonfmt # JSON formatting
              pkgs.dotacat # Colorful terminal output
              pkgs.goreleaser
              pkgs.cuelsp
              pkgs.b3sum
            ]
            ++ pkgs.lib.optional pkgs.stdenv.isLinux [
              pkgs.wild
              pkgs.clang
            ]; # Fast linker (RUST), only works with clang for now

            devshell.startup.shellHook.text = ''
              export RUST_TARGET=$(rustc --version --verbose | grep '^host:' | awk '{print $2}')
              ${hooks.shellHook}
              (
                # Use a lockfile to prevent multiple instances from stomping on Git
                flock -n 9 || exit 1

              ) 9>/tmp/nunu_sync.lock &
            '';
          };
        }
      );

      # Expose devShell as a package for `nix shell` compatibility
      packages = eachSystem (
        { system, ... }:
        {
          default = self.devShells.${system}.default;
        }
      );

      formatter = eachSystem ({ pkgs, ... }: pkgs.nixfmt-rfc-style);
    };
}
