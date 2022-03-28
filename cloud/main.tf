provider "aws" {
    profile = "default"
    region  = "us-east-1"
}

resource "aws_key_pair" "shbot_api" {
    key_name        = "shbot_api_v${var.infra_version}"
    public_key      = "${file("keypairs/keypair.pub")}"
}

terraform {
    # Where we save the infra state info.
    backend "s3" {
        encrypt = true
        bucket  = "shbot"
        region  = "us-east-1"
        key     = "v1"
    }

    required_providers {
        aws = {
            source  = "hashicorp/aws"
            version = "~> 3.27"
        }
    }

    required_version = ">= 0.14.9"
}

# Manages our api server's resources
# (Add more outputs if we need them!)
module "server" {
    source = "./modules/server"

    api_name = "shbot_api"
    subdomain_name = ""
    vpc_id = var.vpc_id
    infra_version = var.infra_version
    enable_green = var.enable_green
    enable_blue = var.enable_blue
    ecr_api_image = "***REMOVED***/shbot_api"
    public_key_name = aws_key_pair.shbot_api.key_name
}
