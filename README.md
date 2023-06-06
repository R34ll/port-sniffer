# Port Sniffer

This repository contains a multi-threaded port sniffer implemented in Rust using the Rayon library. The port sniffer allows you to scan a specified range of ports on a given target host.

## Usage:
./port-sniffer < host > < start_port > < end_port > < num_threads >

- < host >: The target host to scan for open ports.
- < start_port >: The starting port of the range to scan.
- < end_port >: The ending port of the range to scan.
- < num_threads >: The number of threads to use for scanning the ports.

## Example:
./port-sniffer google.com 20 500 16

This command will scan ports 20 to 500 on the host google.com using 16 threads concurrently.

___
Feel free to clone, modify, and use this repository to build your own custom port sniffer tool.
