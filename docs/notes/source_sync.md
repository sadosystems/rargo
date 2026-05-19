I want to steal all the ideas from bonanza (just not the crypto) because I 
__think__ they are good. but I am not actually totally sure I understand it, 
I think The Sync engine in bonanza uses CDC to chunk files, and uses a 
ProllyTree to sync chunks to a CAS... and the protocol has some interesting 
smart stuff where small files / folders can get inlined into a single proto 
message. confusingly though it does not seem like the bonanza protocol does 
a Merkle DAG style negotiation to find a minimal diff... I think the 
brute-force-iest way for me to get this shit in my head is first make sure I 
understand the more basic case of a normal Merkle DAG sync system, then make
sure I understand ProllyTrees THEN come back to the bonanza code.

https://github.com/buildbarn/bonanza
https://wyag.thb.lt/
https://github.com/buildbarn/bonanza/blob/8e8c7aa25603b16fa3e6bd992da0c273d78a5317/docs/filesystem_merkletree.md?plain=1#L161 
https://docs.dolthub.com/architecture/storage-engine/prolly-tree
https://www.dolthub.com/blog/2022-06-27-prolly-chunker/
https://www.dolthub.com/blog/2025-06-03-people-keep-inventing-prolly-trees/
https://github.com/attic-labs/noms/blob/master/doc/intro.md#prolly-tree-construction

I started on this and realized I actually didn't understand how to write proper
rust data structures as well as I thought, so I ended up reading this cool book:
https://rust-unofficial.github.io/too-many-lists/
I worked through it but wrote Btrees instead of linked lists, later I found
this essay from the same author on exactly the topic of writing a Btree in rust 
https://faultlore.com/blah/rust-btree-case/
pretty crazy!