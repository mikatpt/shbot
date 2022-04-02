set shell := ["bash", "-uc"]
set dotenv-load := true
set export

watch:
    cargo watch -x run

down:
    cd cloud && terraform apply -var 'enable_green=false' -var 'enable_blue=false'

blue:
    cd cloud && terraform apply -var 'enable_green=false'

green:
    cd cloud && terraform apply -var 'enable_blue=false'

both:
    cd cloud && terraform apply