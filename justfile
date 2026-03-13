build:
    nix build ".#cross-aarch64-linux"

run: build
    scp ./result/bin/counting-words tegra-1:
    ssh tegra-1 ./counting-words
    ssh tegra-1 "rm -f ./counting-words"
