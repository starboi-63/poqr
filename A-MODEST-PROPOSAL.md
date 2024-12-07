# Post-Onion Quantum-Routing (POQR)

***Pronounced “poker”***

***AKA “Every layer, everywhere, all at once”***

By Tanish Makadia and Alex Khosrowshahi

## An outline of the project you want to implement–what do you want to achieve?

Quantum computers are coming—whether we like it or not.
Governments are storing vastamounts of general internet traffic for
retroactive decryption in anticipation of having these machines perfected
(breaking current encryption schemes like RSA, AES, and ECDH).
If we are to retain our anonymity and privacy,
then we must bolster the security of existing
onion-routing protocols and utilize post-quantum encryption schemes.
Fundamentally, we’d like to create a bare-bones
implementation of *The Onion Router* (TOR) protocol
using Rust on a small virtual network, leveraging the NTRU cryptosystem
developed by Brown Professors Hoffstein, Pipher, and Silverman,
for a post-quantum secure implementation of TOR’s core functionality.

## The key functionality we’d like to achieve

1. Sending packets from one host to another using onion-routing
2. Having the root node which sent the packet unidentifiable to the final hop destination node
3. An executable CLI program allowing users to send and receive messages using our routing/encryption scheme 

## Any stretch goals you think may be difficult but nice to have

At its core, the post-quantum encryption scheme is a stretch goal, as onion routing itself is already a non-trivial project. Additionally, if NTRU proves a significant barrier to our process, we can use cryptosystems with existing implementations such as the Module-Lattice-Based Key-Encapsulation Mechanism. Even further, if all else fails, we can implement boring ol’ onion routing (terrible, we know). 

## Any tools, libraries, or language(s) you intend to use (doesn't need to be a final list)



* Our implementation will be in Rust primarily, as both of us worked in Rust throughout the course.
* We intend to use Rust’s built-in TCP and IP libraries, as we would rather not deal with persistent errors from our personal implementations.
* We’ll likely be using tokio, the asynchronous Rust library, for any asynchronous processing features.
* We’ll use Rust’s subtle crate for constant-time cryptographic operations
* We may use the rustnomial crate for some of the polynomial operations involved in the NTRU implementation
* We’ll also be setting up a virtual network of hosts and proxies using docker containers and TMUX.

## Any open questions you'd like us to help you answer

Some illumination into how to set up virtual networks like those in TCP/IP would be excellent! We want to spend more time actually building the routing protocol rather than doing setup work.

Is this at all feasible? 

