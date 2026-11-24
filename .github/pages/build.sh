#!/usr/bin/env bash
set -euo pipefail

site=${1:?Provide the output directory}
base_path=${2:?Provide the GitHub Pages base path}

mkdir -p "$site"
cp .github/pages/index.html "$site/index.html"

for example in middleware dropdown context_menu; do
    dx build --web --release --example "$example" --locked --debug-symbols false \
        --base-path "$base_path/$example"
    public="target/dx/$example/release/web/public"
    test -s "$public/index.html"
    mkdir -p "$site/$example"
    cp -R "$public/." "$site/$example/"
done
