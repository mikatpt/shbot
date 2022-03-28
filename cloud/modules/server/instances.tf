locals {
    subnets = aws_subnet.api.*.id
    env     = var.enable_green && var.enable_blue ? "split" : (
              var.enable_green ? "green" : "blue")
}

#### IAM Permissions for EC2 instance
# We'll attach this profile to the instances.
# This allows each instance to use the aws CLI without needing to authenticate
resource "aws_iam_instance_profile" "api_profile" {
    name = "${var.api_name}_profile"
    role = aws_iam_role.api.name
}

# Create role
resource "aws_iam_role" "api" {
    name = "${var.api_name}_role"

    assume_role_policy = jsonencode({
        Version = "2012-10-17"
        Statement = [
            {
                Action = "sts:AssumeRole"
                Effect = "Allow"
                Sid    = ""
                Principal = {
                    Service = "ec2.amazonaws.com"
                }
            },
        ]
    })
}

# This policy allows all ECR actions.
resource "aws_iam_role_policy" "ecr_policy" {
  name = "ecr_policy"
  role = aws_iam_role.api.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
            "ecr:*",
            "cloudtrail:LookupEvents"
        ]
        Effect   = "Allow"
        Resource = "*"
      },
    ]
  })
}

# We can scale up availability zones and instances separately
# since the element function wraps indexes
# 
# Ex.
# count = 10
# subnet_id = "${element(local.subnets, count.index)}"
# 
resource "aws_instance" "blue" {
    count                   = var.enable_blue ? 1 : 0
    ami                     = "ami-000722651477bd39b"
    iam_instance_profile    = aws_iam_instance_profile.api_profile.name
    instance_type           = "t2.micro"
    subnet_id               = local.subnets[0]
    vpc_security_group_ids  = [aws_security_group.api.id]
    key_name                = var.public_key_name

    user_data = data.cloudinit_config.api.rendered

    tags = {
        Name                    = "${var.api_name}_blue_${0 + 1}_v${var.infra_version}"
        InfrastructureVersion   = var.infra_version
    }
}

resource "aws_instance" "green" {
    count                   = var.enable_green ? 1 : 0
    ami                     = "ami-000722651477bd39b"
    iam_instance_profile    = aws_iam_instance_profile.api_profile.name
    instance_type           = "t2.micro"
    subnet_id               = local.subnets[1]
    vpc_security_group_ids  = [aws_security_group.api.id]
    key_name                = var.public_key_name

    user_data = data.cloudinit_config.api.rendered

    tags = {
        Name                    = "${var.api_name}_green_${0 + 1}_v${var.infra_version}"
        InfrastructureVersion   = var.infra_version
    }
}
