#### HTTPS Stuff (no clue how any of this works)

# Create SSL certs for HTTPS
resource "aws_acm_certificate" "shbot_api" {
    domain_name         = aws_route53_record.shbot_com_record.fqdn
    validation_method   = "DNS"

    lifecycle {
        create_before_destroy = true
    }
}

# Proves we own the domain
resource "aws_route53_record" "cert_validation" {
    allow_overwrite = true
    name            = tolist(aws_acm_certificate.shbot_api.domain_validation_options)[0].resource_record_name
    records         = [ tolist(aws_acm_certificate.shbot_api.domain_validation_options)[0].resource_record_value ]
    type            = tolist(aws_acm_certificate.shbot_api.domain_validation_options)[0].resource_record_type
    zone_id         = aws_route53_zone.com.zone_id
    ttl             = 60

}

# This tells terraform to cause the route53 validation to happen
resource "aws_acm_certificate_validation" "cert" {
  certificate_arn         = aws_acm_certificate.shbot_api.arn
  validation_record_fqdns = [ aws_route53_record.cert_validation.fqdn ]
}

####

resource "aws_route53_zone" "com" {
    name = "shereebot.com"
}

resource "aws_route53_zone" "net" {
    name = "shereebot.net"
}

# Points traffic from shereebot.com to alb
resource "aws_route53_record" "shbot_com_record" {
    zone_id = aws_route53_zone.com.zone_id
    name    = aws_route53_zone.com.name
    type    = "A"

    alias {
        name                    = aws_lb.shbot_api.dns_name
        zone_id                 = aws_lb.shbot_api.zone_id
        evaluate_target_health  = true
    }
}

# Points traffic from shereebot.net to alb
resource "aws_route53_record" "shbot_net_record" {
    zone_id = aws_route53_zone.net.zone_id
    name = aws_route53_zone.net.name
    type = "A"

    alias {
        name                    = aws_lb.shbot_api.dns_name
        zone_id                 = aws_lb.shbot_api.zone_id
        evaluate_target_health  = true
    }
}

# We need the outputted name servers for our domain; the domain will redirect traffic
# to the name servers instead. We must manually copy these to Google Domain!
output "shbot_com_name_servers" {
    value = aws_route53_zone.com.name_servers
}

output "shbot_net_name_servers" {
    value = aws_route53_zone.net.name_servers
}
