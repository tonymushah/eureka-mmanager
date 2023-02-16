# special eureka (mangadex-desktop-api2)

[![Rust](https://github.com/tonymushah/eureka-mmanager/actions/workflows/rust.yml/badge.svg)](https://github.com/tonymushah/eureka-mmanager/actions/workflows/rust.yml)

## What is this??

It's a ... um ... i don't know what to describe this but it's just a system to download and manage the offline data of the [special-eureka](https://github.com/tonymushah/special-eureka) app. \
It's built in Rust and use [actix](https://actix.rs) for the service

## How it work??

It's basically a Actix server deployed locally in your device.

### Setting dir

When you launch the app, it will verify if a "settings" dir is set, if not, it will generate the settings dir.
Inside this directory, you will find normally those files: 

1. files-dirs.json: The app use this file for "managing your data". There are the keys inside:

    - "data" : is the path where the app store the ressources, by default, it's "data"
    - "chapters" : is the path where the app store the downloaded chapters (it's relative to data directory by the way), by default, it's "chapters"
	- "covers" : is the path where the app store the downloaded covers (relative to data directory by the way), by default, it's "covers"
	- "mangas" : is the path where the app store the downloaded mangas (relative to data directory too), by default, it's "mangas"
	- "history" : is a special directory where the app store the download history

2. server-options.json: used to launch the actix server: 

	- "hostname" : the hostname (it's naturally the IP address), by default, it's "127.0.0.1" (localhost)
	- "port" : the port where the app will be deployed, by default, it's "8145"

## How to use it??

0. Install Rust
1. Clone this repository
2. Run the app by typing in your terminal :

```

cargo run

```

Normally, the app should run on localhost:8145

### API Endpoints

Actually, there are 32 endpoints in this app.\
I will detail them in a near future but there is a Postman collection where you can download
and use
