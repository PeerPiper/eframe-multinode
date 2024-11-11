# justfile for just.systems recipes. 

version := `cat Cargo.toml | grep version | head -n 1 | cut -d '"' -f 2`

# 1. Create a new tag to trigger a release.
# 2. Push the tag to GitHub.
# 3. GitHub Actions will then build and publish the release.
release-tag:
  echo "Releasing version {{version}}"
  git tag -a v{{version}} -m "Release version {{version}}"
  git push origin v{{version}}

web-dev:
  trunk serve --open

native-dev:
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



