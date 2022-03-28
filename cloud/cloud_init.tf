data "template_file" "cloud_init" {
    template = file("cloud_init.yml")

    vars = {
        ecr_img = var.ecr_api_image
    }
}

data "cloudinit_config" "shbot_api" {
    gzip = true
    base64_encode = true
    part {
        filename = "init.cfg"
        content_type = "text/cloud-config"
        content = data.template_file.cloud_init.rendered
    }
}
