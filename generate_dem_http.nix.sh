#!/bin/sh
nix shell nixpkgs#openjdk nixpkgs#fastmod -c ./generate_dem_http.sh
