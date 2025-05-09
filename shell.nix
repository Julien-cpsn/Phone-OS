let
  nixpkgs-esp-dev = builtins.fetchGit {
    url = "https://github.com/mirrexagon/nixpkgs-esp-dev.git";
  };

  pkgs = import <nixpkgs> { overlays = [ (import "${nixpkgs-esp-dev}/overlay.nix") ]; };
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
      esp-idf-full
    ];
    
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH";
}
