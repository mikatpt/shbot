variable "vpc_id" {
    type = string
}

variable "infra_version" {
    default = "1"
}

variable "enable_green" {
    description = "Enable green environment"
    type        = bool
    default     = true
}

variable "enable_blue" {
    description = "Enable blue environment"
    type        = bool
    default     = true
}

variable "ecr_url" {
    type    = string
}
