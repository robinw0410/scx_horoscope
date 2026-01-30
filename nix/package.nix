{ lib, rustPlatform, llvmPackages, pkg-config, elfutils, zlib, libseccomp, }:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "scx_horoscope";
  version = "0.1.0";

  src = ../.;

  cargoHash = "sha256-smzOODMTSy1ISmUfIrC/7DffHB2+dcLx9kAtMAg7JTE=";

  nativeBuildInputs = [ pkg-config rustPlatform.bindgenHook ];
  buildInputs = [ elfutils zlib libseccomp ];

  env = {
    BPF_CLANG = lib.getExe llvmPackages.clang;
    RUSTFLAGS = lib.concatStringsSep " " [
      "-C relocation-model=pic"
      "-C link-args=-lelf"
      "-C link-args=-lz"
    ];
  };

  hardeningDisable = [ "zerocallusedregs" ];

  meta = {
    mainProgram = "scx_horoscope";

    description =
      "An astrological sched_ext scheduler - schedules tasks based on planetary positions";
    longDescription = ''
      A fully functional sched_ext scheduler that makes real CPU scheduling decisions
      based on real-time planetary positions, zodiac signs, and astrological principles.
      This actually loads into the Linux kernel and schedules your system tasks.
      Because if the universe can influence our lives, why not our CPU scheduling too?
    '';

    homepage = "https://github.com/zampierilucas/scx_horoscope";
    license = lib.licenses.gpl2Only;
    platforms = lib.platforms.linux;
  };
})

