#### VPC and subnets

locals {
    # Adjust to add more public subnets everywhere.
    availability_zones  = ["us-east-1a", "us-east-1b"]
}

### Internet access

# We need a gateway to access the internet from public subnets
resource "aws_internet_gateway" "api" {
    vpc_id = var.vpc_id
}

# NAT gateway is required for private subnets to have internet access
resource "aws_eip" "nat_eip" {
    vpc        = true
    depends_on = [aws_internet_gateway.api]
}

# NAT gateway must be created in a public subnet, to connect to internet gateway
# only one per public subnet
resource "aws_nat_gateway" "nat" {
    allocation_id = aws_eip.nat_eip.id
    subnet_id     = aws_subnet.public[0].id
    depends_on    = [aws_internet_gateway.api]
    tags = {
        Name = "shbot-nat"
    }
}

### Public subnets

# TODO: pass in aws_subnet.public_api.*.id to module.server
resource "aws_subnet" "public" {
    count                   = length(local.availability_zones)
    vpc_id                  = var.vpc_id
    availability_zone       = "${element(local.availability_zones, count.index)}"
    map_public_ip_on_launch = true

    # TODO: I don't know how cidr blocks work
    cidr_block              = "10.0.${length(local.availability_zones) * (var.infra_version - 1) +
    count.index}.0/24"

    tags = {
        Name = "${element(local.availability_zones, count.index)} (v${var.infra_version})"
    }
}

# Provide internet access routes for public subnets
resource "aws_route_table" "public" {
    vpc_id = var.vpc_id

    route {
        cidr_block = "0.0.0.0/0"
        gateway_id = aws_internet_gateway.api.id
    }

    tags = {
        Name = "shbot-public-route-table"
    }
}

# Associate all public subnets with the above route table.
resource "aws_route_table_association" "api" {
    count           = length(local.availability_zones)
    subnet_id       = aws_subnet.public[count.index].id
    route_table_id  = aws_route_table.public.id
}

### Private subnets

resource "aws_subnet" "private" {
    vpc_id              = var.vpc_id
    cidr_block          = "10.0.255.0/24"
    availability_zone   = "us-east-1a"
}

# Private route table routes through NAT gateway
resource "aws_route_table" "private" {
    vpc_id = var.vpc_id

    tags = {
        Name = "shbot-private-route-table"
    }

    route {
        cidr_block      = "0.0.0.0/0"
        nat_gateway_id  = aws_nat_gateway.nat.id
    }
}

# Associate private subnet with above route table
resource "aws_route_table_association" "private" {
    subnet_id       = aws_subnet.private.id
    route_table_id  = aws_route_table.private.id
}

# # Add route for private route table to nat gateway
# resource "aws_route" "db" {
#     route_table_id         = aws_route_table.private.id
#     destination_cidr_block = "0.0.0.0/0"
#     nat_gateway_id         = var.nat_gateway_id
# }
