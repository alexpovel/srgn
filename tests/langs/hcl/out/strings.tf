variable "base_domain" {
  type    = string
  default = "example.com"
}

variable "environment__T__" {
  type    = string
  default = "prod"
}

locals {
  # Simple literal string
  project_name = "web-app"

  # String Concatenation with interpolation
  __T__domain_name = "${var.environment__T__}.${var.base_domain}"

  # Multi-line strings
  welcome_text = <<EOT
Welcome to ${local.__T__domain_name}.
This is a multi-line string.
You can format text across multiple lines.
EOT

  # Templated string with interpolation
  templated_greeting = "Hello, ${upper(var.base_domain)} user!"

  # Using string functions
  formatted_string = replace(local.__T__domain_name, ".", "-")
}
