{
  description = "The backend of Bill Keeper, a software to track and help accounting.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      crossPkgs = pkgs.pkgsCross.aarch64-multiplatform.pkgsStatic;

      # Parameterised over a package set `p` (native or cross) so the same
      # definition builds both targets with the correct toolchain/libraries.
      mkApp =
        p:
        let
          arch = p.stdenv.hostPlatform.parsed.cpu.name;
          bin = "bill-keeper-${arch}";
        in
        p.rustPlatform.buildRustPackage {
          pname = "bill-keeper";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with p; [
            pkg-config
          ];
          buildInputs = with p; [
            openssl
          ];

          # e2e needs PostgreSQL + a live server; run it via checks.e2e instead.
          doCheck = false;

          postInstall = ''
            mv $out/bin/bill-keeper $out/bin/${bin}
            ln -s ${bin} $out/bin/bill-keeper
          '';
          meta.mainProgram = "bill-keeper";
        };

      # Hermetic e2e run: boots a throwaway PostgreSQL inside the sandbox.
      mkE2ECheck =
        p:
        p.rustPlatform.buildRustPackage {
          pname = "bill-keeper-e2e";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with p; [
            pkg-config
            postgresql
            poppler-utils
          ];
          buildInputs = [ p.openssl ];

          doCheck = p.stdenv.buildPlatform == p.stdenv.hostPlatform;

          preCheck = ''
            export PGDATA="$TMPDIR/pgdata"
            initdb -D "$PGDATA" -U bill_keeper_testing --auth=trust --no-locale >/dev/null
            pg_ctl -D "$PGDATA" -o "-h 127.0.0.1 -p 5432 -k $TMPDIR" -l "$TMPDIR/pg.log" -w start
            createdb -h 127.0.0.1 -p 5432 -U bill_keeper_testing bill_keeper_testing

            export POSTGRES_HOST=127.0.0.1
            export POSTGRES_PORT=5432
            export POSTGRES_USER=bill_keeper_testing
            export POSTGRES_PASSWORD=bill_keeper_testing
            export POSTGRES_DB=bill_keeper_testing

            # e2e spawns ./target/debug/bill-keeper; buildPhase only makes release.
            cargo build
          '';

          postCheck = ''
            pg_ctl -D "$TMPDIR/pgdata" -w stop || true
          '';

          installPhase = ''
            mkdir -p $out
          '';
        };
    in
    {
      packages.${system} = {
        default = mkApp pkgs;
        arm = mkApp crossPkgs;
      };

      checks.${system} = {
        e2e = mkE2ECheck pkgs;
      };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          clang
          llvmPackages.libclang
          pkg-config
          cmake
          cargo
          rustc
          rustfmt
          clippy
          openssl
          openssl.dev
          poppler-utils
        ];
        shellHook = ''
          export LIBCLANG_PATH=${pkgs.llvmPackages.libclang.lib}/lib
        '';
      };
    };
}
