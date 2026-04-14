resource "random_password" "kennel_webhook_secret" {
    length  = 64
    special = false
}
