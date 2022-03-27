locals {
    subnets = aws_subnet.shbot_api.*.id
}

#### IAM Permissions for EC2 instance
# We'll attach this profile to the instances.
# This allows each instance to use the aws CLI without needing to authenticate
resource "aws_iam_instance_profile" "shbot_api_profile" {
  name = "shbot_api_profile"
  role = aws_iam_role.shbot_api.name
}

# Create role
resource "aws_iam_role" "shbot_api" {
  name = "shbot_api_role"

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
  role = aws_iam_role.shbot_api.id

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
    count                   = var.enable_blue_env ? 1 : 0
    ami                     = "ami-000722651477bd39b"
    iam_instance_profile    = aws_iam_instance_profile.shbot_api_profile.name
    instance_type           = "t2.micro"
    subnet_id               = "${flatten(local.subnets)[0]}"
    vpc_security_group_ids  = [aws_security_group.shbot_api.id]
    key_name                = aws_key_pair.shbot_api.key_name

    user_data = data.cloudinit_config.shbot_api.rendered

    tags = {
        Name                    = "sheree_bot_blue_${0 + 1}_v${var.infra_version}"
        InfrastructureVersion   = var.infra_version
    }
}

resource "aws_instance" "green" {
    count                   = var.enable_green_env ? 1 : 0
    ami                     = "ami-000722651477bd39b"
    iam_instance_profile    = aws_iam_instance_profile.shbot_api_profile.name
    instance_type           = "t2.micro"
    subnet_id               = "${flatten(local.subnets)[1]}"
    vpc_security_group_ids  = [aws_security_group.shbot_api.id]
    key_name                = aws_key_pair.shbot_api.key_name

    user_data = data.cloudinit_config.shbot_api.rendered

    tags = {
        Name                    = "sheree_bot_green_${0 + 1}_v${var.infra_version}"
        InfrastructureVersion   = var.infra_version
    }
}

# for SSH convenience
output "instance_public_ips" {
    value = var.traffic_distribution == "split" ? concat(aws_instance.blue.*.public_ip, aws_instance.green.*.public_ip) : (var.traffic_distribution == "green" ? aws_instance.green.*.public_ip : aws_instance.blue.*.public_ip)
}

# for health check poller
output "instance_ids" {
    value = var.traffic_distribution == "split" ? concat(aws_instance.blue.*.id, aws_instance.green.*.id) : (var.traffic_distribution == "green" ? aws_instance.green.*.id : aws_instance.blue.*.id)
}
