# RGB-121 Library

> :warning: **Warning: this is a work in progress**

RGB121 is an RGB schema for collectibles on bitcoin & lightning.

This repository provides rust library and a command-line utility `rgb121` which
can be used alongside RGB Node to generate and parse RGB121 data (schema, issue
assets, interpret contract information returned by RGB Node).

## Command-line utility

### Install with Docker

#### Build

Clone the repository and checkout to the desired version (here `v0.1.0`):

```console
$ git clone https://github.com/RGB-Tools/rust-rgb121
$ cd rust-rgb121
$ git checkout v0.1.0
```

Build and tag the Docker image:

```console
$ docker build -t rgb121:v0.1.0 .
```

#### Usage

```console
$ docker run rgb121:v0.1.0 --help
```
