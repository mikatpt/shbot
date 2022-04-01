# Shbot

A slack bot written in Rust for fun.

## Getting started

### Requirements
- terraform
- aws-cli
- rust
- just

### Local dev
- Copy `.env.example` to `.env` and add missing variables.
    - `TF_VAR_` variables are only necessary for running deployments.
    - If deploying, also populate a `.env.prod` file.
- `cargo run`
    
## Deployments
- Set up `aws-cli` and authenticate to `us-east-1`

### SSH setup
```bash
#!/bin/bash

# Create pub/priv keypair:
mkdir -p cloud/keypairs
ssh-keygen -f cloud/keypairs/keypair -P ""
mv cloud/keypairs/keypair ~/.ssh/keypair.pem

# Add the following to your ssh config file:
## (the shell script setup is very hacky, make sure `Host` is on line 1 and it'll work)

# ~/.ssh/config
Host 55.555.555.555
    User ubuntu
    IdentityFile ${HOME}/.ssh/keypair.pem
Host 55.5.555.55
    User ubuntu
    IdentityFile ${HOME}/.ssh/keypair.pem
```

### Spinning up architecture
- This will spin up a database and the `blue` env in AWS `us-east-1`.

```bash
source .env

# You can use the just command runner to call terraform
just blue

# Or
cd cloud
terraform apply --var 'enable_green=false'
```

### Scripts
```bash
# SSH to instances:
./scripts/connect api
./scripts/connect db

# Deploy (blue green deployment)
./scripts/deploy
```
