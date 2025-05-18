name := "compiler-builtins"
upstream_repo := "rust-lang/" + name
upstream_url := "https://github.com/" + upstream_repo

josh_port := "42042"
josh_filter := ":/library/compiler-builtins"
josh_url_base := "http://localhost:" + josh_port
josh_cache_dir := cache_directory() / "rust-lang.compiler-builtins-josh"

# Set traps to kill Josh on shell exit or failure, then launch Josh.
start_josh := \
	"trap 'exit $EXIT_CODE' INT TERM\n" + \
	"trap 'EXIT_CODE=$?; kill 0' EXIT\n" +  \
	"josh-proxy --local '" + josh_cache_dir + "' "+ \
		"--remote 'https://github.com' " + \
		"--port '" + josh_port + "' " + \
		"--no-background &\n" + \
	just_executable() + " _ensure-josh-running\n"

update_version_msg := """
	Preparing for merge from rustc
"""
rustc_to_builtins_msg := """
	Merge from rustc
"""

default:
	just --list

# Exit with failure if the working directory has uncommitted files
_ensure-clean:
	#!/bin/bash
	set -eaxo pipefail
	if [ -n "$(git status --untracked-files=no --porcelain)" ]; then
		echo "working directory must be clean"
		exit 1
	fi

# Make sure Josh is reachable.
_ensure-josh-running:
	#!/bin/bash
	set -eaxo pipefail
	for _ in $(seq 1 100); do
		sleep 0.01s

		# Exit with success if we can connect to the port
		if nc -z 127.0.0.1 "{{ josh_port }}"; then
			exit
		fi
	done

	echo "Even after waiting for 1s, josh-proxy is still not available."
	exit 1

# Update this repo with changes from rust-lang/rust
rustc-pull: _ensure-clean
	#!/bin/bash
	set -eaxo pipefail

	commit="$(git ls-remote "{{ upstream_url }}" HEAD | cut -w -f1)"
	josh_url="{{ josh_url_base }}/{{ upstream_repo }}.git@${commit}{{ josh_filter }}.git"

	{{ start_josh }}

	previous_base_commit="$(cat rust-version)"
	if [ "$previous_base_commit" = "$commit" ]; then
		echo "Nothing to pull; commit at $commit"
		exit 1
	fi

	orig_head="$(git rev-parse HEAD)"
	echo "$commit" > "rust-version"
	git commit rust-version --no-verify -m '{{ update_version_msg }}'

	if ! git fetch "$josh_url"; then
		echo "FAILED to fetch new commits, something went wrong."
		echo "(committing the rust-version file has been undone)"
		git reset --hard "$orig_head"
		exit 1
	fi

	num_roots_before="$(git rev-list HEAD --max-parents=0 --count)"
	sha="$(git rev-parse HEAD)"
	git merge FETCH_HEAD --no-verify --no-ff -m '{{ rustc_to_builtins_msg }}'
	new_sha="$(git rev-parse HEAD)"

	if [ "$sha" = "$new_sha" ]; then
		git reset --hard "$orig_head"
		echo "No merge was performed, no changes to pull were found. Rolled back the preparation commit."
		exit 1
	fi

	num_roots="$(git rev-list HEAD --max-parents=0 --count)"

	if [ "$num_roots" -ne "$num_roots_before" ]; then
		echo "Josh created a new root commit. This is probably not the history you want."
		exit 1
	fi

# Create a pull request to rust-lang/rust with changes from this repo
rustc-push github_user branch="builtins-update": _ensure-clean
	#!/bin/bash
	set -eaxo pipefail

	base="$(cat rust-version)"
	branch="{{ branch }}"
	github_user="{{ github_user }}"
	josh_url="{{ josh_url_base }}/{{ github_user }}/rust.git{{ josh_filter }}.git"
	user_rust_url="git@github.com:{{ github_user }}/rust.git"
	
	if [ -z "$RUSTC_GIT" ]; then
		echo "The RUSTC_GIT environment variable must be set"
		exit 1
	fi

	{{ start_josh }}

	(
		# Execute in the rustc directory
		cd "$RUSTC_GIT"

		echo "Preparing $github_user/rust (base: $base)..."
	
		if git fetch "$user_rust_url" "$branch" > /dev/null; then
			echo "The branch '$branch' seems to already exist in '$user_rust_url'. \
				  Please delete it and try again."
			exit 1
		fi

		git fetch "https://github.com/{{ upstream_repo }}" "$base"
		git push "$user_rust_url" "$base:refs/heads/$branch"
	)

	# Do the actual push.
	echo "Pushing changes..."
	git push "$josh_url" "HEAD:$branch"

	# Do a round-trip check to make sure the push worked as expected
	git fetch "$josh_url" "$branch"
	head="$(git rev-parse HEAD)"
	fetch_head="$(git rev-parse FETCH_HEAD)"

	if [ "$head" != "$fetch_head" ]; then
		echo "Josh created a non-roundtrip push! Do NOT merge this into rustc!"
		echo "Expected '$head', got '$fetch_head'."
		exit 1
	fi

	echo "Confirmed that the push round-trips back to {{ name }} properly. Please create a rustc PR:"
	# Open PR with `subtree update` title to silence the `no-merges` triagebot check
	echo "    {{ upstream_url }}/compare/$github_user:$branch?quick_pull=1&title=rustc-dev-guide+subtree+update&body=r?+@ghost"
