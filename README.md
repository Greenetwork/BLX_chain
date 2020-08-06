BLX_chain Claimer module ⚖️

To run on local machine:
- [install substrate](https://substrate.dev/docs/en/tutorials/create-your-first-substrate-chain/)
- clone repo
- cd into BLX_chain/node
- (build it using [--release](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html?highlight=--release#building-for-release) and development key injection into onchain keystore using --features ocw) $ `cargo build --release --features ocw`
- cd up one level to main dir
- (clear any data from previous development node) $ `./target/release/node-template purge-chain --dev`
- (start the development node) $ `./target/release/node-template --dev`
- navigate to [Apps JS](https://polkadot.js.org/apps/#/settings/developer)
- change to local node via node selector in upper left, choose last option for local node
- navigate to the settings -> developer and put the contents of types.json (located in main dir) into the box and save. 
- now explore creating ApnTokens with the claimer pallet accessible via the extrinsics tab 
- this is done via two step process by the claimer pallet 
- extrinsic #1 -> _insertNewTask_ here put in task # (use `1`), basin (use `2`), and apn (use `18102019`), and submit, watch terminal output of chain for confirmation that data is being fetched
- extrinsic #2 -> _emptyTasks_ here submit the extrinsic without input, the offchain worker defaults to attempting to submit a signed transaction so long as there is no task in the queue, so by removing the task we are in a round-about way submitting the offchain worker data back on chain

A new FRAME-based Substrate node, ~~ready for hacking.~~ thats been hacked to pieces!

*  Code needs to be fully commented!  

*  All tests are for sure broken  

*  Needs testing with multiple parties (Accounts), multiple apns (ApnTokens)

*  Not entirely sure who owns the claimed ApnToken, assuming its Alice, but requires more work to confirm

*  Composable tokens likely required, meaning ApnToken (NFT) might need to be able to own another NFT or ERC-20 style token, Annual Allocation. NFT vs ERC-20 token choice will determine if all water in basin is considered the equal or not. 

* Front end idea - [polkadash](https://dotleap.com/polkadash-a-vuejs-dashboard-starter-kit-for-your-substrate-chain/)
  
## What this does right now?:

Creates some *related* data structures for and manually put information into:
* basins - probably broken
* ApnTokens (Digitized Water Rights) - preliminary information store on chain for now
* Annual Allocations for each Apntoken(not sure here yet)

## Directory definitions:
* node - this contains code which runs the node, usually this doesnt get hacked on too hard, mostly left alone
* pallets - this contains the custom pallets (aka modules/crates) which are going to be implemented by the runtime as part of our chain
* scripts - docker stuff and dev env stuff
* runtime - this contains the code which builds the state transitions function (aka the runtime) mostly this gets updated to reflect changes to custom pallets
* types.json - json of defintions which help the front end read (Polkadot Apps JS for now) chain output

Diagram of BLX_network this repo currently contains the __Substrate Claimer Pallet__  

![image](https://drive.google.com/uc?export=view&id=1F6F5cAr8El8iRzhxb95JW2UIjEAhxsSu)

Hackusama - HACKATHON GOALS:
* Contribution to Decentralization and Web 3.0 Friendliness (25%)
 
 Includes how useful the blockchain or tooling is in the Kusama, Polkadot, and Web 3.0 ecosystem as a whole.

* Originality, Creativity, and Innovation (25%)

Includes how new and novel the submission is versus existing technologies.

* Technical Difficulty (25%)

Includes the level of skill or knowledge required to build the parachain or tools.

* User Experience (25%)

Includes how intuitive and understandable the submission is for potential users.