data "template_file" "cloud_init" {
    template = file("./modules/server/cloud_init.yml")

    vars = {
        ecr_img     = var.ecr_api_image
        api_name    = var.api_name
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
