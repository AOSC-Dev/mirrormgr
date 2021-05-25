# apt-gen-list-rs
Utility for generating sources.list for APT according to repository configurations (Rust version)

## Manual
```
apt-gen-list --help
```

## Installation
```
$ cargo build --release
# install -Dvm755 target/release/apt-gen-list /usr/local/bin/apt-gen-list

// install repo data, eg aosc:
$ git clone https://github.com/AOSC-Dev/aosc-os-repository-data.git
# mkdir -pv /usr/local/share/distro-repository-data/
# install -Dvm644 aosc-os-repository-data/* -t /usr/local/share/distro-repository-data/
# install -dv "/usr/share/zsh/functions/Completion/Linux/"
# install -Dvm644 completions/_apt-gen-list "/usr/share/zsh/functions/Completion/Linux/"
# install -dv "/usr/share/fish/completions/"
# install -Dvm644 completions/apt-gen-list.fish "/usr/share/fish/completions/"
# install -dv "/usr/share/bash-completion/completions/"
# install -Dvm644 completions/apt-gen-list.bash "/usr/share/bash-completion/completions/"
```

## Dependencies

Building:
- Rust w/ Cargo
- C compiler
- make (when GCC LTO is used, not needed for Clang)

Runtime:
- Glibc
- OpenSSL
- APT
