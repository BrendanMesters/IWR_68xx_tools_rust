# IWR68xx Tools

**NOTE** This repository is, as of writing, still very much a work in progress and not yet ready for use.
**NOTE** The terms _FMCW_, _mmWave_ and _IWR6843_ might be used interchangably.

This repository holds a set of tools I wrote to be able to work with the IWR68xx series of Frequency Modulated Continuous Wave sensors from [Texas Instruments](https://www.ti.com/sensors/mmwave-radar/overview.html)

I am not legally responsible for any usage of my code, and the usage of this code is at your own risk. Not that I think its bad code, but I want to be sure :-)
Also, if you extend this code, or use it in your own projects, please do credit this repository :-)

The general concept of this project is to have a set of tools which allow for interaction with, and data collection from the IWR6843 mmWave sensors.

I myself work with an IWR6843ISK, and my code is thus designed for this model.

I will try to update this readme with more useful information as time goes on.

## Project structure

The setup for this project (aka the structure and functionality of the `main.rs` file) will be tailored to my personal needs from this project (at least, for now). However, the different functions should be easily adaptable to ones own need.

`file_readers.rs` contains different functions which support the reading of some different configuration files, namely:
- `./settings.toml`, a file describing some basic settings, currently only containing the cfg and data port name, as well as the corresponding baud rate.
- `./config.cfg`, the IWR6843 configuration script, this script will be send to the FMCW to describe what it should do, and to tell it to start working.
- `./tlv_file.dat`, this is **not** a configuration file, but rather a pre-recorded file containing the raw output data from the FMCW, this can also be read in and processed (this code is in `main.rs`, but the specific code might be commented)


`fmcw_manager.rs` holds the `Fmcw` object definition. This is an object which manages communication with the FMCW chip, it also contains the `run` function which is supposed to be ran in a thread. This function will then read the data from the FMCW and publish this to a provided channel.


`tlv_translator.rs` contains the code which parses the raw TLV data returned by the FMCW, as a result, this file has become rather large.
At the top various objects used in the rest of the program are defined. Note how _unions_ are used to quickly and easily parse the raw byte data into, for example, the headers.
The entrance points to the logic are either the `parse_stream` function or the `translate_tlv` function.
Currently only _detected points_ and _range profile_ TLV frames can be parsed. 
If you want to expand this code to parse different types of TLV data then you should do so from the `match` statement in the `parse_frame` function (if you expand on the code, please consider creating a pull request back to this repository :-)  )

