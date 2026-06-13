{ lib, rustPlatform, makeWrapper, curl, cacert }:

rustPlatform.buildRustPackage {
  pname = "mtgbrain";
  version = "0.1.0";

  # Only the bits cargo needs -- keeps the multi-hundred-MB ./data dir out of the store.
  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];

  # `mtgbrain download` shells out to curl; make it available + give it a CA bundle.
  postInstall = ''
    wrapProgram $out/bin/mtgbrain \
      --prefix PATH : ${lib.makeBinPath [ curl ]} \
      --set-default SSL_CERT_FILE ${cacert}/etc/ssl/certs/ca-bundle.crt
  '';

  meta = {
    description = "MTGJSON cards in a queryable SQLite DB for LLM-driven card search";
    license = lib.licenses.mit;
    mainProgram = "mtgbrain";
  };
}
