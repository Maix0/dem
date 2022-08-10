# Discord Emoji Manager (and stickers)

## Builing and running
First you need to generate the http client for the webapp. You need to have the server running so go to `dem-server` and use `cargo run`, then do run `genrate_dem_http.sh` (or `generate_dem_http.nix.sh` for nixos).

after this, you will do `trunk build` inside the `dem-client` folder and you will be able to run the webapp.
The webapp will be served at `http://localhost:8000`
