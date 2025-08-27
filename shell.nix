let
  nixpkgs-esp-dev = builtins.fetchGit {
    url = "https://github.com/mirrexagon/nixpkgs-esp-dev.git";
  };

  pkgs = import <nixpkgs> { overlays = [ (import "${nixpkgs-esp-dev}/overlay.nix") ]; };

  esp-idf = pkgs.esp-idf-full.override {
    rev = "v5.4.1";
    sha256 = "sha256-5hwoy4QJFZdLApybV0LCxFD2VzM3Y6V7Qv5D3QjI16I=";
  };
in
  pkgs.mkShell rec {
    buildInputs = with pkgs; [
      # Standard development tools
      pkg-config
      flex
      gperf
      bison
      cmake
      ninja
    
      # Libraries needed for ESP-IDF and libclang
      openssl
      libffi
      libusb1
      libclang
      stdenv.cc.cc.lib  # This provides libstdc++.so.6
      zlib
      ncurses

      ldproxy
      espup
      espflash 
      esp-idf
    ];
    
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH";

    /*
    shellHook = ''
      export PATH="${esp-idf}/python-env/bin"
    '';*/
}
