variable "enable_green_env" {
    description = "Enable green environment"
    type        = bool
    default     = true
}

variable "enable_blue_env" {
    description = "Enable blue environment"
    type        = bool
    default     = true
}

variable "ecr_image_shbot_api" {
    default = "***REMOVED***/shbot_api"
}

locals {
    traffic_dist_map = {
        blue = {
            blue = 100
            green = 0
        }
        # We aren't using this right now, not sure we need canary deploys
        split = {
            blue = 50
            green = 50
        }

        green = {
            blue = 0
            green = 100
        }
    }
}

variable "traffic_distribution" {
    description = "Levels of traffic distribution"
    type        = string
}
