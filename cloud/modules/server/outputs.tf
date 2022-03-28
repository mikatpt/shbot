# We need the outputted name servers for our domain; the domain will redirect traffic
# to the name servers instead. We must manually copy these to Google Domain!
output "name_servers" {
    value = aws_route53_zone.com.name_servers
}

# for SSH convenience
output "instance_public_ips" {
    value = local.env == "split" ? concat(aws_instance.blue.*.public_ip,
    aws_instance.green.*.public_ip) : (local.env == "green" ? aws_instance.green.*.public_ip : aws_instance.blue.*.public_ip)
}

# for health check poller
output "instance_ids" {
    value = local.env == "split" ? concat(aws_instance.blue.*.id, aws_instance.green.*.id) : (local.env == "green" ? aws_instance.green.*.id : aws_instance.blue.*.id)
}

# output "subnet_ids" {
#     value = aws_subnet.api.*.id
# }
