# How to use the MassFinder
First, you need to build it using this command:
`cargo build --bin chlorophyte-tshock-checker --release`
##### That will put the executable in target/release/chlorophyte-tshock-checker. You have to run it as root, as the custom TCP stack requires raw socket access. Alternatively, you can run `sudo setcap cap_net_admin target/release/chlorophyte-mass-finder` to make it work without root.
Then you can run it:
`sudo target/release/chlorophyte-tshock-checker chlorophyte-mass-finder/chlorophyte_mass_finder_results.txt`to `chlorophyte_mass_finder_results.txt`