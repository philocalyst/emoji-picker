package release

import "cue.dev/x/goreleaser"

goreleaser.#Project & {
	version:      2
	project_name: "moji"

	// I miss Nix
	before: hooks: [
		"rustup default stable",
		"cargo install --locked cargo-zigbuild",
		"cargo install cargo-bundle",
	]

	// Build everywhere
	builds: [{
		builder: "rust"
		binary:  "moji"
		dir:     "src"
		flags: ["--release"]
		targets: [
			"x86_64-apple-darwin",
			"aarch64-apple-darwin",
			"x86_64-unknown-linux-gnu",
			"aarch64-unknown-linux-gnu",
			"x86_64-pc-windows-gnu",
		]
	}]

	// For bundling
	after: [
		{
			cmd: "cargo bundle --release --format osx"
			if:  "{{ eq .Runtime.Goos \"darwin\" }}"
		},
		{
			cmd: "cargo bundle --release --format deb"
			if:  "{{ eq .Runtime.Goos \"linux\" }}"
		},
		{
			cmd: "cargo bundle --release --format msi"
			if:  "{{ eq .Os \"windows\" }}"
		},
	]

	archives: [{
		formats: ["tar.gz"]
		name_template: "{{ .ProjectName }}_{{ .Os }}_{{ .Arch }}"
		files: ["README.md"]
	}]

	// Ensure the bundles endup there
	extra_files: [
		{glob: "target/release/bundle/**/*.app"},
		{glob: "target/release/bundle/**/*.deb"},
		{glob: "target/release/bundle/**/*.msi"},
	]

	checksum: {
		name_template: "SHA256.sum"
		algorithm:     "sha256"
	}

	snapshot: name_template: "{{ .Tag }}-next"

	release: disable: true
}
