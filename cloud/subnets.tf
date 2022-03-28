# VPC and Subnets

# If planning on adding more, also adjust instances.tf
locals {
    availability_zones  = ["us-east-1a", "us-east-1b"]
}

resource "aws_subnet" "shbot_api" {
    count                   = length(local.availability_zones)
    vpc_id                  = var.vpc_id
    availability_zone       = "${element(local.availability_zones, count.index)}"
    cidr_block              = "10.0.${length(local.availability_zones) * (var.infra_version - 1) +
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
    count           = length(local.availability_zones)
    subnet_id       = aws_subnet.shbot_api[count.index].id
    route_table_id  = aws_route_table.shbot_api_route.id
}
