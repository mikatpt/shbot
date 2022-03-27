variable "infra_version" {
    default = "1"
}

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
