#!/bin/bash
echo "Basin Logix - Digital Water Rights. project installer, using substrate rc-4 or greater"
# Copyright 2015-2020 Parity Technologies (UK) Ltd.\

read -p "
######################################################################
This executable has been tested (working) on linux distro Ubuntu 18 only

You will need Substrate rc-4 or greater installed for the chain
as well as yarn for the front end...

curl https://getsubstrate.io -sSf | bash -s -- --fast and /n
https://github.com/substrate-developer-hub/substrate-front-end-template 

Press enter to continue if you have these installed
######################################################################
"

mkdir chain
cd ./chain

echo "Cloning BLX_chain to current working directory"
git clone https://github.com/Greenetwork/BLX_chain.git

cd ./BLX_chain/node 
echo "compiling BLX_chain this may take a while...~5-30 minutes"
cargo build --release --features ocw

cd ../../../

mkdir ./front-end
cd ./front-end

echo "Cloning BLX_frontend to current working directory"
git clone https://github.com/Greenetwork/BLX_frontend

echo "starting chain in background"
../chain/BLX_chain/target/release/node-template --dev &

echo "starting front end"
cd ./BLX_frontend
yarn install
yarn start
