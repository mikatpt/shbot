resource "aws_security_group" "shbot_api" {
    description = "ShereeBot API"
    vpc_id      = var.vpc_id
    name        = "shbot_api_v${var.infra_version}"

    tags = {
        Name = "ShereeBot API (v${var.infra_version})"
    }
}

resource "aws_security_group_rule" "shbot_api_inbound" {
    type                = "ingress"
    security_group_id   = aws_security_group.shbot_api.id
    from_port           = -1
    to_port             = 0
    protocol            = "-1"

    cidr_blocks         = ["0.0.0.0/0"]
}

resource "aws_security_group_rule" "shbot_api_outbound" {
    type = "egress"
    security_group_id   = aws_security_group.shbot_api.id
    from_port           = -1
    to_port             = 0
    protocol            = "-1"

    cidr_blocks         = ["0.0.0.0/0"]
}

