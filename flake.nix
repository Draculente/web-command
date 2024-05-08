{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.75.0";
          packageFun = import ./Cargo.nix;
        };

      in rec {
        packages = {
          wsh = (rustPkgs.workspace.wsh {});
          default = packages.wsh;
        };
        nixosModules = {
          default = ({ config, lib, pkgs, ... }:

            with lib;

            {
              options = {
                services.wsh = {
                  enable = mkEnableOption "Enable the WSH service";
                  port = mkOption {
                    type = types.int;
                    default = 8012;
                    description = "Port for the WSH service";
                  };
                  mirror = {
                    enable = mkOption {
                      type = types.bool;
                      default = true;
                      description = "Whether to enable mirror mode or use a local configuration";
                    };
                    url = mkOption {
                      type = types.str;
                      default = "https://wsh.draculente.eu";
                      description = "URL for the mirror";
                    };
                    configureNginx = mkEnableOption "Configure Nginx for the mirror";
                  };
                };
              };

              config = mkIf config.services.wsh.enable {
                systemd.services.wsh = {
                    description = "WSH Service";
                    wantedBy = [ "multi-user.target" ];
                    serviceConfig = {
                    ExecStart = "${packages.wsh}/bin/wsh"; #I want wsh to be the package exported by flake.nix
                    Environment = [
                        "WEBCOMMAND_PORT=${toString config.services.wsh.port}"
                        "WEBCOMMAND_CONFIG=${config.services.wsh.mirror.url}"
                        "WEBCOMMAND_HOST_MODE=${toString config.services.wsh.mirror.enable}"
                    ];
                    };
                    after = [ "network.target" ];
                };
              };
            }
          );
        };
      }


    );
}
