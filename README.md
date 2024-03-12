# Chlorophyte

<p>
    <img src="https://github.com/Paddyk45/chlorophyte/blob/main/assets/logo.png" height=100 alt="Chlorophyte">
</p>

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
##### That will put the executable in target/release/chlorophyte-mass-finder. You have to run it as root, as the custom TCP stack requires raw socket access. Alternatively, you can run `sudo setcap cap_net_admin target/release/te-mass-finder` to make it work without root.
Then you can run it:
`sudo target/release/chlorophyte-mass-finder 0.0.0.0/0:7777`
##### This will scan almost the entire internet on port 7777.
##### You can also specify ranges instead of subnets, and a port range instead of a single port.
##### You can specify a comma-seperated list of ranges
Some example ranges:
- `1.0.0.0/16:13337`: Will scan the 1.0.0.0/16 subnet on port 13337
- `1.10.0.0-1.10.10.128:7777`: Will scan all IP-addresses between 1.10.0.0 and 1.10.10.128 on port 7777
- `1.0.0.0:7000-8000`: Will scan 1.0.0.0 on ports between 7000 and 8000
- `1.0.0.0/4:7777,2.0.0.0:7000-8000`: Will scan the 1.0.0.0/4 subnet on port 7777 and 2.0.0.0 on ports 7000-8000
- `1.0.0.0/24:7000-9000,11.0.10.0-11.12.0.128:7777-7800`: Will scan the 1.0.0.0/24 subnet on ports 7000-9000 and all IP-addresses between 11.0.10.0 and 11.12.0.128 on ports between 7777 and 7800

At the end of the scan, all found servers will be written to `chlorophyte_mass_finder_results.txt`

## Licenses
All crates starting with chlorophyte- are licensed under the GLWTPL. For more info, see LICENSE.
All crates starting with matscan- are part of [matscan](https://github.com/mat-1/matscan) and are licensed under the [matscan license](https://raw.githubusercontent.com/mat-1/matscan/master/LICENSE)

## Thanks
- [mat](https://github.com/mat-1) for letting me "borrow" small amounts of code from [matscan](https://github.com/mat-1/matscan) and helping me a bit
- [The Wireshark software](https://www.wireshark.org) for helping me debug my stupid errors
- [Some 10-year-old protocol documentation](https://seancode.com/terrafirma/net.html) for some protocol packet stuff
- [Nikolan](https://github.com/nikolan123) for temporarily running the scanner for me
- [Vero](https://github.com/veronoicc/) for helping me
