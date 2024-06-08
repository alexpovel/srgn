variable "base_domain" {
  type    = string
  default = "example__T__.com"
}

variable "environment__T__" {
  type    = string
  default = "__T__prod"
}

locals {
  # Simple literal string
  project_name = "web-app__T__"

  # String Concatenation with interpolation
  __T__domain_name = "__T__${var.environment__T__}__T__.__T__${var.base_domain}__T__"

  # Multi-line strings
  welcome_text = <<EOT
__T__Welcome to ${local.__T__domain_name}.
This is a multi-line string.__T__
You can format text across multiple lines.__T__
EOT

  # Templated string with interpolation
  templated_greeting = "Hello, __T__${upper(var.base_domain)} user!"

  # Using string functions
  formatted_string = replace(local.__T__domain_name, ".__T__", "-")
}
