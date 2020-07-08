BLX_chain Claimer module ⚖️

A new FRAME-based Substrate node, ~~ready for hacking.~~ thats been hacked to pieces!

*  Code needs to be fully commented!  

*  All tests are for sure broken  

*  Issues with parsing .geojson from off-chainworker (github fetch) persist due to [geojson rust crate](https://crates.io/crates/geojson) NOT having a no-std feature, potential work around using cargo nightly cargo-features [resolver](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html?highlight=nightly#resolver) as alluded to in the [off-chain worker recipe](https://substrate.dev/recipes/3-entrees/off-chain-workers/http-json.html).  

*  Next steps are to link off-chainworker results with on-chain storage (APN --> ApnToken)  

*  On-chain storage fetching RPC calls need to be fleshed out, only some work at the moment  

*  Needs testing with multiple parties (Accounts), multiple apns (ApnTokens)

*  Composable tokens likely required, meaning ApnToken (NFT) might need to be able to own another NFT or ERC-20 style token, Annual Allocation. NFT vs ERC-20 token choice will determine if all water in basin is considered the equal or not. 

* Front end [idea](https://morioh.com/p/bf71bb815161)  
  
## What this does right now?:
Does not compile :)

Creates some *related* data structures for and manually put information into:
* basins
* ApnTokens (Digitized Water Rights)
* Annual Allocations (not sure here yet)

It also, 

* Uses off-chainworker to fetch json from Github ~~and dumps to screen~~ broke this a while back

* Most extrinsics (txns) accessible via polkadot apps js UI 

## Directory definitions:
* node - this contains code which runs the node, usually this doesnt get hacked on too hard, mostly left alone
* pallets - this contains the custom pallets (aka modules/crates) which are going to be implemented by the runtime as part of our chain
* scripts - docker stuff and dev env stuff
* runtime - this contains the code which builds the state transitions function (aka the runtime) mostly this gets updated to reflect changes to custom pallets

HACKATHON GOALS:
* Contribution to Decentralization and Web 3.0 Friendliness (25%)
Includes how useful the blockchain or tooling is in the Kusama, Polkadot, and Web 3.0 ecosystem as a whole.

* Originality, Creativity, and Innovation (25%)

Includes how new and novel the submission is versus existing technologies.

* Technical Difficulty (25%)

Includes the level of skill or knowledge required to build the parachain or tools.

* User Experience (25%)

Includes how intuitive and understandable the submission is for potential users.