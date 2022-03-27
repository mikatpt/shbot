# VPC and Subnets

variable "vpc_id" {
    default = "***REMOVED***"
}

# If planning on adding more, also adjust instances.tf
locals {
    subnet_count        = 2
    availability_zones  = ["us-east-1a", "us-east-1b"]
}

resource "aws_subnet" "shbot_api" {
    count                   = local.subnet_count
    vpc_id                  = var.vpc_id
    availability_zone       = "${element(local.availability_zones, count.index)}"
    cidr_block              = "10.0.${local.subnet_count * (var.infra_version - 1) +
    count.index}.0/24"
    map_public_ip_on_launch = true

    tags = {
        Name = "${element(local.availability_zones, count.index)} (v${var.infra_version})"
    }
}

# Manually expose this vpc to the internet.
resource "aws_internet_gateway" "shbot_api" {
    vpc_id = var.vpc_id
}

resource "aws_route_table" "shbot_api_route" {
    vpc_id = var.vpc_id

    route {
        cidr_block = "0.0.0.0/0"
        gateway_id = aws_internet_gateway.shbot_api.id
    }
}

resource "aws_route_table_association" "shbot_api" {
    subnet_id = "${aws_subnet.shbot_api[0].id}"
    route_table_id = aws_route_table.shbot_api_route.id
}
