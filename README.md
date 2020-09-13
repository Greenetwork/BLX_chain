# BLX_chain Claimer module ⚖️

## To run on local machine (other methods not yet supported):
- $ `curl https://getsubstrate.io -sSf | bash -s -- --fast`
- install [yarn](https://yarnpkg.com/getting-started)
- clone this repo $ `git clone https://github.com/Greenetwork/BLX_chain`
- add executable permissions to shell script with $ `chmod +x basin_logix_run.sh`
- compile and run chain and start front end with $ `./basin_logix_run.sh`
- make sure to clear the chain's development data when necessary with $ `BLX_chain/target/release/node-template purge-chain --dev`

## Interacting with BLX_chain
demo is here https://youtu.be/aiY_yprYokA starts at 1:48
- this is done via two step process by the claimer pallet 
- extrinsic #1 -> _insertNewTask_ here put in the apn (use `18102019`), and submit, watch terminal output of chain for confirmation that data is being fetched
- extrinsic #2 -> _emptyTasks_ here submit the extrinsic without input, the offchain worker defaults to attempting to submit a signed transaction so long as there is no task in the queue, so by removing the task we are in a round-about way submitting the offchain worker data back on chain

- the data is now onchain, it is not associated with any account, future work will tie it to a particular account so that there is ownership

## odds and ends
- if not using the shell script (build chain using [--release](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html?highlight=--release#building-for-release) and development key injection into onchain keystore using --features ocw) $ `cargo build --release --features ocw`


A new FRAME-based Substrate node, ~~ready for hacking.~~ thats been hacked to pieces!

*  Code needs to be fully commented!  

*  All tests are for sure broken  

*  Needs testing with multiple parties (Accounts), multiple apns (ApnTokens)
   *  will come after DB implementation

*  ApnToken now has owner field, other pallets will need to leverage this owner field and the method of tracking ownership

*  ~~Composable tokens likely required, meaning ApnToken (NFT) might need to be able to own another NFT or ERC-20 style token, Annual Allocation. NFT vs ERC-20 token choice will determine if all water in basin is considered the equal or not.~~
   *  ApnToken's (NFT) index = Apn is also used at the index for `WaterBalanceBySuperApns`
   *  the WaterBalance "tokens/balance" will be fungible on certain interfaces (voting) but non-fungible in others (trading)

* Front end idea - ~~[polkadash](https://dotleap.com/polkadash-a-vuejs-dashboard-starter-kit-for-your-substrate-chain/)~~ moved forward with a [modified Substrate-fronend-template](https://github.com/Greenetwork/BLX_frontend) (react)
  
## What this does right now?:

Creates some *related* data structures for and manually put information into:
* basins - depricated for now
* ApnTokens (Digitized Water Rights) - preliminary information store on chain for now
* WaterBalance for each Apntoken(not sure here yet)

## Directory definitions:
* node - this contains code which runs the node, usually this doesnt get hacked on too hard, mostly left alone
* pallets - this contains the custom pallets (aka modules/crates) which are going to be implemented by the runtime as part of our chain
* scripts - docker stuff and dev env stuff
* runtime - this contains the code which builds the state transitions function (aka the runtime) mostly this gets updated to reflect changes to custom pallets
* types.json - json of defintions which help the front end read (Polkadot Apps JS for now) chain output

Diagram of BLX_network this repo currently contains the __Substrate Claimer Pallet__  

![image](https://drive.google.com/uc?export=view&id=1W2lAew1JVMTzfz0HG1TjvkLSxI0cJ1aJ)

