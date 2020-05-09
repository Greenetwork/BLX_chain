## Preliminary Substrate based blockchain for tokenizing water allocations
based off [substrate-node-template](https://github.com/substrate-developer-hub/substrate-node-template) + [recipes](https://substrate.dev/recipes/introduction.html)

---
### Pallets  
[info](https://www.substrate.io/kb/runtime/pallets)
* __`pallets/allocator`__ - creator of allocations, utilizing data created by [data_logix](https://github.com/Greenetwork/Basin_Logix/tree/master/data_logix) collected via offchain workers (IPFS), will start with allocation dependent on apn area, transition to `pallets/water_master`
* __`pallets/water_master`__ - DAO style governance module, basin logic imported, quadratic voting by stakeholders in system on allocation proposals, multi-year 
  * basin logic: offchain allocaiton optimization via linear algebra or similar  
* __`pallets/extraction`__ - burning of water allocation tokens based on well pumping
* __`pallets/market`__ - water market allocation decentralized exchange via substrate contract pallet (ink! smart contracts), involved in the selling/buying of AnnualAllocation balances or futures of yet realized/cemented Annual Allocations

---
### Data Flow
__allocator related__
* WebApp User input (APNs owned + wells owned) --> conduit.py --> .json uploaded to IPFS
* WebApp User input (APNs owned + wells owned) -async-> allocator pallet + .json from IPFS --> allocations/tokenized water rights
* Well based oracle (gallons extracted) --> extraction pallet --> modified allocations/tokenized water  

__market related__

---
### Data Locations  
__On Chain__  
* `struct` AnnualAllocation - apn (Vec array or hash?), balance (acre-feet, unsigned int), year of allocation: annual total_allocation (map, u32:u32), reasoning for annual allocation (governance voting outcome transaction hash)  
* `struct` Non Fungbile Token - apn (Vec array or hash?), metadata (IPFS hash), AnnualAllocation (struct)  

__IPFS__  
* .json - apn, geojson, basin, Groundwater Sustainability Agency, groundwater rights, total acres, owner  

__Oracles__
* (wells = network nodes = oracles) cumulative gallons pumped via telemetry  

__Web App__  
* apns owned, wells owned

---
### Missing Utility  
* outside investment mechanism (non land owners)
  * Maybe just BLX stock
  * could Tokenize entire basins as a fundrasing mechanism for stakeholders 
* exact business/money making mechanics i.e. where to extract value from?  
  * what does crypto-economics enable us to create, this is transfering value from physical realm to digital realm
  * consulting style contract based
  * transaction fees on market
  * subscription service
  * retain portion of tokens from Tokenizing entire basins

`
