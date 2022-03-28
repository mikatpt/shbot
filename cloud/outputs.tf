# We need the outputted name servers for our domain; the domain will redirect traffic
# to the name servers instead. We must manually copy these to Google Domain!
output "name_servers" {
    value = module.server.name_servers
}
# for SSH convenience
output "instance_public_ips" {
    value = module.server.instance_public_ips
}

# for health check poller
output "instance_ids" {
    value = module.server.instance_ids
}

output "env" {
    value = var.enable_green && var.enable_blue ? "split" : (
            var.enable_green ? "green" : "blue")
}
