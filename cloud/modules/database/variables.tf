variable "infra_version" {
    type = string
}

variable "vpc_id" {
    type = string
}

variable "public_key_name" {
    description = "name for saved public key; for SSH'ing"
    type        = string
}

variable "private_subnet_id" {
    type = string
}
