resource "aws_lb" "shbot_api" {
    name                = "shbot-api-v${var.infra_version}"
    internal            = false
    load_balancer_type  = "application"
    subnets             = aws_subnet.shbot_api.*.id
    security_groups     = [aws_security_group.shbot_api.id]
}

# Primary listener. Directs traffic to the correct target group depending on our deploy.
# Weight options are blue|green|split
resource "aws_lb_listener" "shbot_api_https" {
    load_balancer_arn   = aws_lb.shbot_api.arn
    port                = 443
    protocol            = "HTTPS"
    certificate_arn     = aws_acm_certificate.shbot_api.arn

    default_action {
        type = "forward"
        forward {

            target_group {
                arn     = aws_lb_target_group.blue.arn
                weight  = lookup(local.traffic_dist_map[var.traffic_distribution], "blue", 100)
            }

            target_group {
                arn     = aws_lb_target_group.green.arn
                weight  = lookup(local.traffic_dist_map[var.traffic_distribution], "green", 0)
            }

            stickiness {
                enabled     = false
                duration    = 1
            }
        }

    } 
}

# Redirect all http traffic to https
resource "aws_lb_listener" "shbot_api_http" {
    load_balancer_arn   = aws_lb.shbot_api.arn
    port                = 80
    protocol            = "HTTP"

    default_action {
        type = "redirect"
        redirect {
            port        = "443"
            protocol    = "HTTPS"
            status_code = "HTTP_301"
        }
    } 
}

# Blue resources
resource "aws_lb_target_group" "blue" {
    name        = "shbot-api-blue"
    port        = 80
    protocol    = "HTTP"
    vpc_id      = var.vpc_id

    health_check {
        path                = "/_health"
        port                = 7070
        healthy_threshold   = 3
        unhealthy_threshold = 3
        timeout             = 6
        interval            = 30
        matcher             = "200"
    }
}

resource "aws_lb_target_group_attachment" "blue" {
    count               = length(aws_instance.blue)
    target_group_arn    = aws_lb_target_group.blue.arn
    target_id           = aws_instance.blue[count.index].id
    port                = 80
}

# Green resources
resource "aws_lb_target_group" "green" {
    name        = "shbot-api-green"
    port        = 80
    protocol    = "HTTP"
    vpc_id      = var.vpc_id

    health_check {
        path                = "/_health"
        port                = 7070
        healthy_threshold   = 3
        unhealthy_threshold = 3
        timeout             = 6
        interval            = 30
        matcher             = "200"
    }
}

resource "aws_lb_target_group_attachment" "green" {
    count               = length(aws_instance.green)
    target_group_arn    = aws_lb_target_group.green.arn
    target_id           = aws_instance.green[count.index].id
    port                = 80
}
