# How to use the MassFinder
First, you need to build it using this command:
`cargo build --bin chlorophyte-mass-finder --release`
##### That will put the executable in target/release/chlorophyte-mass-finder. You have to run it as root, as the custom TCP stack requires raw socket access. Alternatively, you can run `sudo setcap cap_net_admin target/release/chlorophyte-mass-finder` to make it work without root.
Then you can run it:
`sudo target/release/chlorophyte-mass-finder 0.0.0.0/0:7777`
##### This will scan almost the entire internet on port 7777. You can also specify ranges instead of subnets, and a port range instead of a single port. You can specify a comma-seperated list of ranges too
Some example ranges:
- `1.0.0.0/16:13337`: Will scan the 1.0.0.0/16 subnet on port 13337
- `1.10.0.0-1.10.10.128:7777`: Will scan all IP-addresses between 1.10.0.0 and 1.10.10.128 on port 7777
- `1.0.0.0:7000-8000`: Will scan 1.0.0.0 on ports between 7000 and 8000
- `1.0.0.0/4:7777,2.0.0.0:7000-8000`: Will scan the 1.0.0.0/4 subnet on port 7777 and 2.0.0.0 on ports 7000-8000
- `1.0.0.0/24:7000-9000,11.0.10.0-11.12.0.128:7777-7800`: Will scan the 1.0.0.0/24 subnet on ports 7000-9000 and all IP-addresses between 11.0.10.0 and 11.12.0.128 on ports between 7777 and 7800
##### By default, the pps (packets per second) are throttled to 50000, but you can turn that up or down by providing the pps after the range: `sudo target/release/chlorophyte-mass-finder 0.0.0.0/0:7777 100000`
At the end of the scan, all found servers will be written to `chlorophyte_mass_finder_results-<timestamp>.txt`