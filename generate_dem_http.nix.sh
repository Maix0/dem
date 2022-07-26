#!/bin/sh
nix shell nixpkgs#openjdk -c ./generate_dem_http.sh
