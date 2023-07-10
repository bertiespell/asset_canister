# Asset_canister

This canister is an example of how assets can be stored on the IC, which is compatible with the Signals Dapp https://signalsicp.com/. Signals allows users to save assets into their own user-controlled canister by configuring the canister ID on a user's profile page. In order to configure your own canister, so that you control the image and video data in your own canister, you can follow the instructions here.


## Build Locally

This project uses React on the front-end and Rust for the backend canisters. As well as Node, you'll need to follow the setup steps here:

```bash
# Use correct node version in each terminal
nvm use 
# Set env varibles for dfx 
export DFX_NETWORK=local
# Starts the replica, running in the background
dfx start --clean 

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:8000?canisterId={asset_canister_id}`.

Additionally, if you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8081`, proxying API requests to the replica at port 8000.

## During development to test again with clean state

```bash
dfx stop
dfx start --clean
dfx deploy
npm start
```

You'll maybe want to remove any problem packages from the dfx.json (for instance installing ledger isn't working yet)

 dev (in Signals) this now falls back to 8081

## Releases and Deployment

We use the `staging` icp address to test out changes before they reach the main address. You can check the canister IDs that are used in `canister_ids.json`.

If you want to deploy to staging, you need to first make sure:

1. You're on main and up-to-date with the remote
2. That you have the latest changes
3. That you don't have any uncommitted changes

Otherwise the script will fail. To deploy to staging:

```bash
node scripts/deploy-to-staging.js
```

This will increment the patch version in `src/beacon_frontend/src/version.json` by 1. It then creates and pushes a commit to main with a tag, then releases this to staging by doing `dfx deploy --network staging`. You should refrain from running `dfx deploy --network staging` outside of the release script, as this won't tag the version properly (and makes knowing what is deployed difficult).

To deploy to mainnet

```bash
dfx deploy --network ic
```

Note that by default all canisters will be deployed, if you want to deploy just one, say the front-end, you must specify it like so:

```bash
dfx deploy --network ic beacon_frontend
```

## Security Audits

The Rust package uses `cargo-audit` to ensure packages are secure. This requires open-ssl to be installed and available in your environment. If you see errors relating to this try:

```bash
brew install pkg-config
brew install openssl
```

Then you may need to export the env vars as per the logged output instructions:

```bash
If you need to have openssl@3 first in your PATH, run:
  echo 'export PATH="/usr/local/opt/openssl@3/bin:$PATH"' >> ~/.zshrc

For compilers to find openssl@3 you may need to set:
  export LDFLAGS="-L/usr/local/opt/openssl@3/lib"
  export CPPFLAGS="-I/usr/local/opt/openssl@3/include"

For pkg-config to find openssl@3 you may need to set:
  export PKG_CONFIG_PATH="/usr/local/opt/openssl@3/lib/pkgconfig"
```

For packages that need a security update, this can be done automatically using `cargo install cargo-audit --features=fix`
