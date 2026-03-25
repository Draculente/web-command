{
  rustPlatform,
  pkg-config,
  openssl,
}:
rustPlatform.buildRustPackage (finalAttrs:
let 
  meta = builtins.fromTOML (builtins.readFile "${finalAttrs.src}/Cargo.toml");
in  {
  pname = meta.package.name;
  version = meta.package.version;

  src = ./.;

  cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
})
