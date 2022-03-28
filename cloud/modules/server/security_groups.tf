resource "aws_security_group" "api" {
    description = "ShereeBot API"
    vpc_id      = var.vpc_id
    name        = "${var.api_name}_v${var.infra_version}"

    tags = {
        Name = "${var.api_name} (v${var.infra_version})"
    }
}

resource "aws_security_group_rule" "api_inbound" {
    type                = "ingress"
    security_group_id   = aws_security_group.api.id
    from_port           = -1
    to_port             = 0
    protocol            = "-1"

    cidr_blocks         = ["0.0.0.0/0"]
}

resource "aws_security_group_rule" "api_outbound" {
    type = "egress"
    security_group_id   = aws_security_group.api.id
    from_port           = -1
    to_port             = 0
    protocol            = "-1"

    cidr_blocks         = ["0.0.0.0/0"]
}

