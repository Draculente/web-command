{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils"; nixpkgs.follows =
    "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system; overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.75.0"; packageFun = import ./Cargo.nix;
        };

      in rec {
        packages = {
          wsh = (rustPkgs.workspace.wsh {}); default = packages.wsh;
        }; nixosModules = {
          default = ({ config, lib, pkgs, ... }:

            with lib;

            {
              options = {
                services.wsh = {
                  enable = mkEnableOption "Enable the WSH service";
                  port = mkOption {
                    type = types.int; default = 8012; description =
                    "Port for the WSH service";
                  }; host_mode = mkOption {
                    type = types.str; default = "mirror"; description =
                    "Mode for the WSH service: 'mirror' or 'local'";
                  }; mirror = {
                    url = mkOption {
                      type = types.str;
                      default = "https://wsh.draculente.eu";
                      description = "URL for the mirror";
                    };
                  }; configFile = mkOption {
                    type = types.str;
                    default = "/example/path/config.toml";
                    description = "path for a configuration toml file";
                  };
                };
              };

              config = mkIf config.services.wsh.enable {
                systemd.services.wsh = {
                    description = "WSH Service";
                    wantedBy= [ "multi-user.target" ];
                    after = [ "network.target" ];
                    serviceConfig = {
                    ExecStart = "${packages.wsh}/bin/wsh";
                    Environment = [
                        "WEBCOMMAND_PORT=${toString config.services.wsh.port}"
                        ''WEBCOMMAND_CONFIG=${
                            if
                                config.services.wsh.host_mode == "mirror"
                            then
                                config.services.wsh.mirror.url
                            else
                                config.services.wsh.configFile
                        }''
                        "WEBCOMMAND_HOST_MODE=${toString(config.services.wsh.host_mode == "local" )}"
                      ];
                    };
                };
              };

            }
          );
        };
      }
    );
}
