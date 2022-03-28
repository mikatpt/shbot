resource "aws_security_group" "api" {
    description = "ShereeBot API"
    vpc_id      = var.vpc_id
    name        = "${var.api_name}_v${var.infra_version}"

    ingress {
        description = "Allow all incoming traffic"
        from_port   = 0
        to_port     = 0
        protocol    = "-1"
        cidr_blocks = ["0.0.0.0/0"]
    }

    egress {
        description = "Allow all outgoing traffic"
        from_port   = 0
        to_port     = 0
        protocol    = "-1"
        cidr_blocks = ["0.0.0.0/0"]
    }

    tags = {
        Name = "${var.api_name} (v${var.infra_version})"
    }
}
