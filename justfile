release version:
	sed -i 's/^\s*version\s*=\s*".*"\s*$/version = "{{ version }}"/' Cargo.toml
	cargo update
	git add Cargo.toml Cargo.lock
	git commit -m 'chore: release {{ version }}'
	git tag '{{ version }}'
	git push
	git push --tag
