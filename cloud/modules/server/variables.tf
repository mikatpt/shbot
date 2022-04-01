variable "api_name" {
    description = "The name of your api, used for instance/alb naming"
    type        = string
}

variable "subdomain_name" {
    description = "`foo` in foo.bar.com"
    type        = string
}

variable "vpc_id" {
    type = string
}

variable "infra_version" {
    type = string
}

variable "enable_green" {
    description = "Enable green environment"
    type        = bool
}

variable "enable_blue" {
    description = "Enable blue environment"
    type        = bool
}

variable "ecr_url" {
    description = "ECR URL to pull from"
    type        = string
}

variable "public_key" {
    type = string
}

variable "public_key_name" {
    description = "name for saved public key; for SSH'ing"
    type        = string
}

variable "public_subnet_ids" {
    type = list
}
