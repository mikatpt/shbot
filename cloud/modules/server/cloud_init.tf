data "template_file" "cloud_init" {
    template = file("./modules/server/cloud_init.yml")

    vars = {
        ecr_url     = var.ecr_url
        api_name    = var.api_name
        public_key  = var.public_key
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
