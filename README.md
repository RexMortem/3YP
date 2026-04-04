# 3YP
Third Year Project: Yet Another Probabilistic Programming Language (YAPPL). 

## Trying the Project

If you don't want to build and run the project, you should be able to play around with the language at [https://yappl.containers.uwcs.co.uk/](https://yappl.containers.uwcs.co.uk/) (unless the container has crashed or Portainer has decided to brick itself).

## Locally Testing the Project

For targeting a specific file:
```sh
cargo run Sample/Passing/BasicAddOutput.txt
```

For running the web interface locally:
```sh
cargo run -- --web
```
And then visit `http://localhost:8080`.

## Build and Run w/ Docker (Recommended, for Web Interface)

For if you want to locally build and run the project (though you can just use the locally testing instructions immediately above):

```sh
docker build -t yappl .
```

And then to run: 
```sh
docker run -p 8080:8080 yappl
```
*(Keep in mind that Docker should be running while you execute the above commands, or you'll get an error about the daemon not running)*

## Build and Run w/o Docker (Not Recommended)

You can either obtain an `exe` of the most recent release of the project from the GitHub page, or you can build your own with Rust and Cargo.

1) Install Rust and Cargo - guide here: https://doc.rust-lang.org/cargo/getting-started/installation.html

2) Run `cargo build --release`

3) Your executable should be found at the relative path `target/release/third-year-project` 

4) You should have an executable or program of some sort at this stage. If you are using an executable, test by running:

```sh
./path-to-exe Sample/Passing/MulAddOutput.txt
```

This should run the interpreter on a simple example of a syntactically and semantically correct YAPPL program.

## Hosting the project on Portainer
*This is mostly for me (Ed) if I forget how to do this*

When you push this project, the GitHub Action workflow `docker-publish.yml` should trigger which builds and pushes the docker image to the GitHub registry. After pushing, you should see the image at `https://ghcr.io/rexmortem/3yp:latest`.

Then, you should:
1) Go to [Portainer](https://portainer.uwcs.co.uk) 
2) Go to the UWCS Public Portainer environment (or whatever equivalent exists at the time)
3) Go to "Container"
4) Add Container
5) Map Additional Port, with Host being some random unique port number and Container being `8080` e.g. Host=`5421` and Container=`8080`
6) Go to Advanced Container Settings (maybe at bottom of page)
7) Go to Env -> Add an Environment Variable
8) Add VIRTUAL_HOST=yappl
9) Deploy Container
10) Should be accessible at [https://yappl.containers.uwcs.co.uk/](https://yappl.containers.uwcs.co.uk/) 