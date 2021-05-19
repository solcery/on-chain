# How to work with Solana on Windows

Solana has Github repo with simple (?!) "Hello, world" project, which could be useful for initial setup.

Solana test network won't work on Windows, such as specific rust compiler, so we have to use Linux/macOS.

## Virtial machine installation

Installing VM is much simplier than setting up pure dual boot system, so we will use Oracle's VirtualBox.

Download VirtualBox and install it:
https://www.virtualbox.org/wiki/Downloads

Download Ubuntu Desktop image:
https://ubuntu.com/download/desktop

Follow this instruction to setup VM with Ubuntu.
https://brb.nci.nih.gov/seqtools/installUbuntu.html
Default size of HDD space and RAM for VM is 10GB and 1GB accordingly. I suggest thrice the size.

## Setting up Ubuntu

The following text is written with assumption that you aren't well experienced with Unix-based systems.
We need 2 applications:
1) Terminal, accessible by clicking left top corner and typing "terminal"
2) File manager, by default placed on the left system bar

Right now we need the terminal

Install curl. It's necessary for some installers we will run and useful per se.
`sudo snap install curl`

We will need C linker, because Rust doesn't have it's own.
`sudo apt install build-essential`

Next stop is npm, packet manager for javascript also acting as a pipeline manager for example project
`sudo apt install npm `

## Rust installatioon
Run these two commands in terminal
`sudo snap install rustup --classic`
`rustup default stable`

Checking that installation is succesfull:
`rustup --version`
`cargo --version`

## Solana devkit installation

https://docs.solana.com/cli/install-solana-cli-tools
Basic installer won't work with VM. For now we'll use manual installation. 
Follow the part of instuction titled as "Download Prebuilt Binaries"

Download and unpack the archive with `tar jxf` command or via double-clicking on downloaded file.
Move solana-release folder somewhere. I placed it right at Home, so the full path became /home/teuzet/solana-release

Now we need to add this folder to system PATH variable to make solana apps executable. There are two ways to do it
1) Each time we open new terminal window run this command `PATH=/home/%username/solana-release/bin:$PATH`
2) Open /home/%username/.bashrc with Vim `vi /home/%username/.bashrc` nad put the same command somewhere near the end.

I suggest the second one, but Vim could be an awful experience if you have never used it.

## Setting up Solana

Run these two command in terminal.

`solana config set --url localhost`
`solana-keygen new`

## Launching Solana local network

Open new terminal window and run this:

`solana-test-validator`
That will run main Solana process. After this command terminal window will be tied to it and won't be usable anymore.

(Optional) In another terminal window run:
`solana logs `
That will run a new process to wath your local chain logs. Could be interestring.

## Trying to compile and run example-helloworld pproject

On this stage we are already able to compile the rust part of the project manually, using `cargo build-bpf` command in the project folder. But we'll use suggested pipelines on npm.

Open new terminal window and process to root folder of example-helloworld project.

`cd %projectRoot/projects/example-helloworld-master`
where %projectRoot is a folder containing this file.

First of all, we need to install all client-side dependencies. It can be achieved by running
`npm install`
Npm will install everything described in package.json file in current folder.

When it's done, we can compile the backend source code with:
`npm run build:program-rust`

Successful compilation will result in message with such text:
`To deploy this program:
$ solana deploy %path_to_program`

Copypaste this command here (without "$"), and Solana will start deployement. Wait until you see message, containing program ID. When you see program ID, you can try launching client:

`npm run start`
