variable "vpc_id" {
    default = "***REMOVED***"
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

variable "ecr_api_image" {
    default = "***REMOVED***/shbot_api"
}
