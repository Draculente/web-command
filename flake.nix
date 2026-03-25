{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs, ... }:
    {
      overlay = final: prev: {
        wsh = final.callPackage ./default.nix { };
      };
      packages = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (system: {
        default =
          (import nixpkgs {
            inherit system;
            overlays = [ self.overlay ];
          }).wsh;
      });
      homeManagerModule = (
        {
          config,
          lib,
          pkgs,
          ...
        }:

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
              host_mode = mkOption {
                type = types.enum [ "mirror" "local" ];
                default = "mirror";
                description = "Mode for the WSH service: 'mirror' or 'local'";
              };
              mirror = {
                url = mkOption {
                  type = types.str;
                  default = "https://wsh.draculente.eu";
                  description = "URL for the mirror";
                };
              };
              configFile = mkOption {
                type = types.path;
                default = "/example/path/config.toml";
                description = "path for a configuration toml file";
              };
              package = mkOption {
                type = types.package;
                default = pkgs.wsh;
                description = "WSH package to use";
              };
            };
          };

          config = mkIf config.services.wsh.enable {
            systemd.user.services.wsh = {
              Install.WantedBy = [ "default.target" ];
              Unit = {
                After = [ "network.target" ];
                Description = "Web Command Service";
              };
              Service = {
                ExecStart = "${config.services.wsh.package}/bin/wsh";
                Environment = [
                  "WEBCOMMAND_PORT=${toString config.services.wsh.port}"
                  "WEBCOMMAND_CONFIG=${
                    if config.services.wsh.host_mode == "mirror" then
                      config.services.wsh.mirror.url
                    else
                      config.services.wsh.configFile
                  }"
                  "WEBCOMMAND_HOST_MODE=${if config.services.wsh.host_mode == "local" then "true" else "false"}"
                ];
              };
            };
          };
        }
      );
    };
}
