# justfile for just.systems recipes. 

# use just.systems variable for the ollama version
# https://github.com/ollama/ollama/releases 
ollama_version := "v0.5.7"

version := `cat Cargo.toml | grep version | head -n 1 | cut -d '"' -f 2`

# 1. Create a new tag to trigger a release.
# 2. Push the tag to GitHub.
# 3. GitHub Actions will then build and publish the release.
release-tag:
  echo "Releasing version {{version}}"
  git tag -a v{{version}} -m "Release version {{version}}"
  git push origin v{{version}}

build-wits:
 for dir in crates/*; do \
    if ([ -d $dir/wit ] && [ -f $dir/src/bindings.rs ]); then \
     cargo component build --manifest-path=$dir/Cargo.toml --target wasm32-unknown-unknown --release; \
   fi \
 done

web-dev: build-wits
  trunk serve --open

native-dev: build-wits
  cargo run

# -c: Clears the screen before each run
# -q: Suppresses output from cargo watch itself
watch-dev:
  cargo watch -c -q -x 'run'

# Simultaneously run the web and native development environments.
dev: 
  just watch-dev & just web-dev

update-remote:
  git submodule update --recursive --remote

# build the ./crates/submodules/peerpiper/crates/peerpiper-server into a binary 
# and copy it to the ./bin directory 
build-peerpiper: update-remote
  cargo build --release --manifest-path ./crates/submodules/peerpiper/crates/peerpiper-server/Cargo.toml
  cp ./crates/submodules/peerpiper/target/release/peerpiper-server ./bin/peerpiper-server

build: build-wits build-peerpiper

check: build-wits
  ./check.sh

check32:
  cargo check --target wasm32-unknown-unknown

force: build-wits
  cargo run --bin force-build-wasm-bins

# This is called from github/workflows, if you change this name, change that file too
install_ollama_linux:
  echo "Installing ollama on Linux"
  # wget https://github.com/jmorganca/ollama/releases/download/v0.1.20/ollama-darwin 
  wget https://github.com/ollama/ollama/releases/download/{{ollama_version}}/ollama-linux-amd64.tgz
  # Now that we've downloaded the tarball, we need to extract it 
  mkdir -p ollama_files
  tar -xvzf ollama-linux-amd64.tgz --directory ollama_files
  # Make the binary executable 
  chmod +x ollama_files/bin/ollama
  # remove the tar 
  rm ollama-linux-amd64.tgz

install_ollama_macos:
  echo "Installing ollama on Mac"
  wget https://github.com/jmorganca/ollama/releases/download/{{ollama_version}}/ollama-darwin
  chmod +x ollama-darwin

  # needs this specific name?
  mv ollama-darwin src/ollama/ollama-aarch64-apple-darwin

install_ollama_windows:
  echo "Installing ollama on Windows"
  curl -L -O -o . "https://github.com/ollama/ollama/releases/download/{{ollama_version}}/ollama-windows-amd64.zip"
  unzip ollama-windows-amd64.zip

  # needs this specific name?
  mv ollama.exe src/ollama/ollama-x86_64-pc-windows-msvc.exe
  # mv all the *.dll files too, they can keep the same name
  mv *.dll src/ollama/

