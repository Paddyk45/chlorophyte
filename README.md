# Chlorophyte

## What is Chlorophyte?
Chlorophyte is program that scans a good portion of the internet for Terraria v1.4.4.9 servers.

## Current state
The mass-finder is in a working state.
Development of the rescanner and the part that gets the server information is planned but has not been started.

## Usage
##### Note: Chlorophyte only works under Linux and Mac
Before running the scanner, you need to enter this command:
`sudo iptables -A INPUT -p tcp -m multiport --dports 61000:65000 -j DROP`
##### This will make the OS ignore all TCP packets received on ports 61000-65000. This is required because chlorophyte uses a TCP stack separate from the OSes one. If the OS receives a TCP packet without having a connection with that IP, it will send an RST packet. This command makes the OS ignore all connections on this port so that doesn't happen.
After you entered the command, you build the mass finder with the following command:
`cargo build --bin chlorophyte-mass-finder --release`
##### That will put the executable in target/release/te-mass-finder. You have to run it as root, as the custom TCP stack requires raw socket access. Alternatively, you can run `sudo setcap cap_net_admin target/release/te-mass-finder` to make it work without root.

At the end of the scan, all found servers will be printed.

## Licenses
All crates starting with chlorophyte- are licensed under the GLWTPL. For more info, see LICENSE.
All crates starting with matscan- are part of [matscan](https://github.com/mat-1/matscan) and are licensed under the [matscan license](https://raw.githubusercontent.com/mat-1/matscan/master/LICENSE)

## Thanks
- [mat](https://github.com/mat-1) for letting me "borrow" small amounts of code from [matscan](https://github.com/mat-1/matscan) and helping me a bit
- [The Wireshark software](https://www.wireshark.org) for helping me debug my stupid errors
- [Some 10-year-old protocol documentation](https://seancode.com/terrafirma/net.html) for some protocol packet stuff
- [Veronoicc](https://github.com/veronoicc/) for helping me
