data "template_file" "cloud_init" {
    template = file("./modules/database/cloud_init.yml")

    vars = {
        postgres_pass = sensitive(file("./modules/database/.env"))
    }
}

data "cloudinit_config" "api" {
    gzip = true
    base64_encode = true
    part {
        filename = "init.cfg"
        content_type = "text/cloud-config"
        content = data.template_file.cloud_init.rendered
    }
}

resource "aws_instance" "db" {
    ami                     = "ami-000722651477bd39b"
    instance_type           = "t2.micro"
    subnet_id               = var.private_subnet_id
    vpc_security_group_ids  = [aws_security_group.db.id]
    key_name                = var.public_key_name

    user_data = data.cloudinit_config.api.rendered

    tags = {
        Name = "mikatpt_postgres_v${var.infra_version}"
        InfrastructureVersion   = var.infra_version
    }

    lifecycle {
        # prevent_destroy = true
    }
}

resource "aws_security_group" "db" {
    name        = "PostgreSQL"
    description = "Allow SSH and PostgreSQL inbound traffic"
    vpc_id      = var.vpc_id

    ingress {
        description = "SSH"
        from_port   = 22
        to_port     = 22
        protocol    = "tcp"
        cidr_blocks = ["0.0.0.0/0"]
    }

    ingress {
        description = "Postgres"
        from_port   = 0
        to_port     = 5432
        protocol    = "tcp"
        cidr_blocks = ["0.0.0.0/0"]
    }

    egress {
        description = "Allow outbound internet access"
        from_port   = 0
        to_port     = 0
        protocol    = "-1"
        cidr_blocks = ["0.0.0.0/0"]
    }

    tags = {
        Name = "mikatpt_postgres (v${var.infra_version})"
    }
}

output "db_ip" {
    value = aws_instance.db.private_ip
}
