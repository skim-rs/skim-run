default:
  just --list

release version: check fmt
	#!/bin/sh
	prev_tag=$(git tag -l | tail -n1)
	sed -i 's/^\s*version\s*=\s*".*"\s*$/version = "{{ version }}"/' Cargo.toml
	cargo update
	git add Cargo.toml Cargo.lock
	git commit -m 'chore: release {{ version }}'
	git tag '{{ version }}'
	git push
	git push --tag
	gh release create {{ version }} --title {{ version }} --notes "$(git log $prev_tag..{{version}} --pretty='format:%s (by %an)')"

check:
  cargo check
  cargo check --all-features
  cargo check --no-default-features --features apps
  cargo check --no-default-features --features calc
  cargo check --no-default-features --features hyprland
  cargo check --no-default-features --features systemd

fmt:
  cargo fmt --all