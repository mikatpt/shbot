#### HTTPS Stuff (no clue how any of this works)

locals {
    subdomain = var.subdomain_name == "" ? "" : "${var.subdomain_name}."
}

# Create SSL certs for HTTPS
resource "aws_acm_certificate" "com" {
    domain_name         = aws_route53_record.com_record.fqdn
    validation_method   = "DNS"

    lifecycle {
        create_before_destroy = true
    }
}

# Proves we own the domain
resource "aws_route53_record" "cert_validation" {
    allow_overwrite = true
    name            = tolist(aws_acm_certificate.com.domain_validation_options)[0].resource_record_name
    records         = [ tolist(aws_acm_certificate.com.domain_validation_options)[0].resource_record_value ]
    type            = tolist(aws_acm_certificate.com.domain_validation_options)[0].resource_record_type
    zone_id         = aws_route53_zone.com.zone_id
    ttl             = 60
}

# This tells terraform to cause the route53 validation to happen
resource "aws_acm_certificate_validation" "cert" {
    certificate_arn         = aws_acm_certificate.com.arn
    validation_record_fqdns = [ aws_route53_record.cert_validation.fqdn ]
}

####

# Apex domain
resource "aws_route53_zone" "com" {
    name    = "shereebot.com"
}

# Points traffic from *.shereebot.com to alb
resource "aws_route53_record" "com_record" {
    zone_id = aws_route53_zone.com.zone_id
    name    = aws_route53_zone.com.name
    type    = "A"

    alias {
        name                    = aws_lb.api.dns_name
        zone_id                 = aws_lb.api.zone_id
        evaluate_target_health  = true
    }
}

# Later add subdomain support?
