# Chlorophyte

<p>
    <img src="https://github.com/Paddyk45/chlorophyte/blob/main/assets/logo.png" height=100 alt=":3">
</p>

## What is Chlorophyte?
Chlorophyte is program that scans a good portion of the internet for Terraria v1.4.4.9 servers.

## Current state
The mass finder is in a working state.
Development of the rescanner and the part that gets the server information is planned but has not been started.

## Setup
##### Because Chlorophyte is written in Rust, you need to install [rustup](https://rustup.rs) if it's not already installed.
Before running any program that is part of Chlorophyte, you need to enter this command:
`sudo iptables -A INPUT -p tcp -m multiport --dports 61000:65000 -j DROP`
You will need to rerun the command after a reboot.
##### This will make the OS ignore all TCP packets received on ports 61000-65000. This is required because Chlorophyte uses a TCP stack separate from the OSes one. If the OS receives a TCP packet without having a connection with that IP, it will send an RST packet. This command makes the OS ignore all connections on this port so that doesn't happen.

## Usage
Check the `USAGE.md`s of the crates for info on how to use them.
##### Note: Chlorophyte only works under Linux and probably Mac


## Licenses
All crates starting with chlorophyte- are licensed under the GLWTPL. For more info, see LICENSE.

All crates starting with matscan- are part of [matscan](https://github.com/mat-1/matscan) and are licensed under the [matscan license](https://raw.githubusercontent.com/mat-1/matscan/master/LICENSE)

## Thanks
- [mat](https://github.com/mat-1) for letting me "borrow" small amounts of code from [matscan](https://github.com/mat-1/matscan) and helping me a bit
- [The Wireshark software](https://www.wireshark.org) for helping me debug my stupid errors
- [Some 10-year-old protocol documentation](https://seancode.com/terrafirma/net.html) for some protocol packet stuff
- [Nikolan](https://github.com/nikolan123) for temporarily running the scanner for me
- [Vero](https://github.com/veronoicc/) for helping me
