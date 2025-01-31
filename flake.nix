{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        config.allowUnfree = true;
      };
    in
    {
      devShells."x86_64-linux".default = pkgs.mkShell {

        buildInputs = with pkgs; [
          pkg-config
        ];

        packages = with pkgs; [
          cargo
          rustc
          openssl
        ];

        shellHook = ''
          export DIRENV='rust'
        '';
      };
    };
}
